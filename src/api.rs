//! All network calls. Everything runs as the signed-in user with their own login token,
//! so the database's Row Level Security decides what is allowed.
//! Stock-changing writes go through the same database functions the mobile app uses
//! (`add_transaction`, `delete_transaction`, `adjust_stock`) so stock and money stay in step.

use crate::config::{SUPABASE_KEY, SUPABASE_URL};
use crate::model::*;
use crate::store;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

pub type Res<T> = Result<T, String>;

const PAGE: usize = 1000;

#[derive(Clone)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub email: String,
    pub uid: String,
}

pub struct Client {
    agent: ureq::Agent,
    session: Mutex<Option<Session>>,
    remember: AtomicBool,
}

/// Percent-encodes everything except unreserved characters (for values in a query string).
pub fn enc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

fn describe(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            let msg = serde_json::from_str::<Value>(&body).ok().and_then(|v| {
                ["message", "msg", "error_description", "error"]
                    .iter()
                    .find_map(|k| v.get(*k).and_then(|m| m.as_str()).map(|s| s.to_string()))
            });
            msg.unwrap_or_else(|| format!("The server refused the request (HTTP {code})"))
        }
        ureq::Error::Transport(t) => format!("Could not reach the server. Check your internet. ({t})"),
    }
}

impl Client {
    pub fn new() -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build();
        Client { agent, session: Mutex::new(None), remember: AtomicBool::new(false) }
    }

    pub fn set_remember(&self, on: bool) {
        self.remember.store(on, Ordering::Relaxed);
    }

    fn token(&self) -> Res<String> {
        self.session
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.access_token.clone())
            .ok_or_else(|| "You are signed out. Sign in again.".to_string())
    }

    fn store_session(&self, v: &Value, fallback_email: &str) -> Res<Session> {
        let access = v.get("access_token").and_then(|x| x.as_str()).ok_or("Sign-in returned no token")?;
        let refresh = v.get("refresh_token").and_then(|x| x.as_str()).ok_or("Sign-in returned no refresh token")?;
        let user = v.get("user");
        let email = user.and_then(|u| u.get("email")).and_then(|e| e.as_str()).unwrap_or(fallback_email);
        let uid = user.and_then(|u| u.get("id")).and_then(|e| e.as_str()).ok_or("Sign-in returned no user")?;
        let s = Session {
            access_token: access.into(),
            refresh_token: refresh.into(),
            email: email.into(),
            uid: uid.into(),
        };
        *self.session.lock().unwrap() = Some(s.clone());
        if self.remember.load(Ordering::Relaxed) {
            store::save(&store::Saved { email: s.email.clone(), refresh_token: s.refresh_token.clone() });
        }
        Ok(s)
    }

    fn token_request(&self, grant: &str, body: Value, fallback_email: &str) -> Res<Session> {
        let url = format!("{SUPABASE_URL}/auth/v1/token?grant_type={grant}");
        let resp = self.agent.post(&url).set("apikey", SUPABASE_KEY).send_json(body).map_err(describe)?;
        let v: Value = resp.into_json().map_err(|e| e.to_string())?;
        self.store_session(&v, fallback_email)
    }

    pub fn sign_in(&self, email: &str, password: &str) -> Res<Session> {
        self.token_request("password", json!({ "email": email, "password": password }), email)
    }

    pub fn restore(&self, saved: &store::Saved) -> Res<Session> {
        self.token_request("refresh_token", json!({ "refresh_token": saved.refresh_token }), &saved.email)
    }

    /// Creates an account. Returns `Ok(Some(session))` when signed in straight away, `Ok(None)` when the
    /// email must be confirmed first. A database trigger creates the profile row.
    pub fn sign_up(&self, name: &str, email: &str, password: &str) -> Res<Option<Session>> {
        let url = format!("{SUPABASE_URL}/auth/v1/signup");
        let resp = self
            .agent
            .post(&url)
            .set("apikey", SUPABASE_KEY)
            .send_json(json!({ "email": email, "password": password, "data": { "displayName": name } }))
            .map_err(describe)?;
        let v: Value = resp.into_json().map_err(|e| e.to_string())?;
        if v.get("access_token").is_some() {
            Ok(Some(self.store_session(&v, email)?))
        } else {
            Ok(None)
        }
    }

    fn refresh(&self) -> Res<()> {
        let (rt, email) = {
            let g = self.session.lock().unwrap();
            let s = g.as_ref().ok_or("You are signed out. Sign in again.")?;
            (s.refresh_token.clone(), s.email.clone())
        };
        self.token_request("refresh_token", json!({ "refresh_token": rt }), &email)?;
        Ok(())
    }

    pub fn sign_out(&self) {
        *self.session.lock().unwrap() = None;
        store::clear();
    }

    /// Runs a request with the current token; if it has expired, refreshes once and retries.
    fn authed<F>(&self, f: F) -> Res<ureq::Response>
    where
        F: Fn(&ureq::Agent, &str) -> Result<ureq::Response, ureq::Error>,
    {
        let t = self.token()?;
        match f(&self.agent, &t) {
            Ok(r) => Ok(r),
            Err(ureq::Error::Status(401, _)) => {
                self.refresh()?;
                let t = self.token()?;
                f(&self.agent, &t).map_err(describe)
            }
            Err(e) => Err(describe(e)),
        }
    }

    fn get_json<T: DeserializeOwned>(&self, path_and_query: &str, range: Option<(usize, usize)>) -> Res<T> {
        let url = format!("{SUPABASE_URL}/rest/v1/{path_and_query}");
        let resp = self.authed(|a, t| {
            let mut req = a.get(&url).set("apikey", SUPABASE_KEY).set("Authorization", &format!("Bearer {t}"));
            if let Some((from, to)) = range {
                req = req.set("Range-Unit", "items").set("Range", &format!("{from}-{to}"));
            }
            req.call()
        })?;
        resp.into_json().map_err(|e| e.to_string())
    }

    /// PostgREST returns at most 1000 rows per request; this pages through all of them.
    fn get_all<T: DeserializeOwned>(&self, path_and_query: &str) -> Res<Vec<T>> {
        let mut all: Vec<T> = Vec::new();
        let mut from = 0;
        loop {
            let page: Vec<T> = self.get_json(path_and_query, Some((from, from + PAGE - 1)))?;
            let n = page.len();
            all.extend(page);
            if n < PAGE {
                break;
            }
            from += PAGE;
        }
        Ok(all)
    }

    fn post_json(&self, path: &str, body: &Value) -> Res<Value> {
        let url = format!("{SUPABASE_URL}/rest/v1/{path}");
        let resp = self.authed(|a, t| {
            a.post(&url)
                .set("apikey", SUPABASE_KEY)
                .set("Authorization", &format!("Bearer {t}"))
                .set("Prefer", "return=minimal")
                .send_json(body.clone())
        })?;
        Ok(resp.into_json().unwrap_or(Value::Null))
    }

    // ---- reads ----------------------------------------------------------

    pub fn profile(&self, uid: &str) -> Res<Profile> {
        let rows: Vec<Profile> = self.get_json(&format!("users?uid=eq.{}&select=*", enc(uid)), None)?;
        rows.into_iter().next().ok_or_else(|| "Your profile was not found.".to_string())
    }

    pub fn settings(&self) -> Res<Settings> {
        let rows: Vec<Settings> = self.get_json(&format!("settings?id=eq.appConfig&select={SETTINGS_COLUMNS}"), None)?;
        Ok(rows.into_iter().next().unwrap_or_default())
    }

    pub fn transactions(&self, owner: &str, mode: &str) -> Res<Vec<Transaction>> {
        self.get_all(&format!(
            "transactions?userId=eq.{}&mode=eq.{}&select=*&order=date.desc,id.asc",
            enc(owner),
            enc(mode)
        ))
    }

    pub fn products(&self, owner: &str) -> Res<Vec<Product>> {
        let mut items: Vec<Product> =
            self.get_all(&format!("products?userId=eq.{}&select=*&order=name.asc,id.asc", enc(owner)))?;
        items.sort_by_key(|p| p.name.to_lowercase());
        Ok(items)
    }

    pub fn movements(&self, owner: &str, product_id: &str) -> Res<Vec<StockMovement>> {
        self.get_all(&format!(
            "stockMovements?productId=eq.{}&userId=eq.{}&select=*&order=date.desc,id.asc",
            enc(product_id),
            enc(owner)
        ))
    }

    // ---- writes ---------------------------------------------------------

    pub fn add_transaction(&self, owner: &str, mode: &str, creator_name: &str, tx: &NewTransaction) -> Res<()> {
        let mut p = serde_json::to_value(tx).map_err(|e| e.to_string())?;
        if let Some(o) = p.as_object_mut() {
            o.insert("userId".into(), json!(owner));
            o.insert("createdByName".into(), json!(creator_name));
            o.insert("mode".into(), json!(mode));
        }
        self.post_json("rpc/add_transaction", &json!({ "p": p })).map(|_| ())
    }

    pub fn delete_transaction(&self, id: &str) -> Res<()> {
        self.post_json("rpc/delete_transaction", &json!({ "p_id": id })).map(|_| ())
    }

    pub fn add_product(&self, owner: &str, creator_id: &str, creator_name: &str, p: &NewProduct) -> Res<()> {
        let mut v = serde_json::to_value(p).map_err(|e| e.to_string())?;
        if let Some(o) = v.as_object_mut() {
            o.insert("userId".into(), json!(owner));
            o.insert("createdBy".into(), json!(creator_id));
            o.insert("createdByName".into(), json!(creator_name));
        }
        self.post_json("products", &v).map(|_| ())
    }

    pub fn adjust_stock(&self, product_id: &str, kind: &str, quantity: f64, note: &str, creator_name: &str) -> Res<()> {
        self.post_json(
            "rpc/adjust_stock",
            &json!({
                "p_product_id": product_id,
                "p_type": kind,
                "p_quantity": quantity,
                "p_note": note,
                "p_creator_name": creator_name,
            }),
        )
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_query_values() {
        assert_eq!(enc("a b&c"), "a%20b%26c");
        assert_eq!(enc("safe-_.~"), "safe-_.~");
    }
}
