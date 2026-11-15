//! Remembers the signed-in user between launches (refresh token only, never the password).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Saved {
    pub email: String,
    pub refresh_token: String,
}

/// Per-user data folder (`%LOCALAPPDATA%\QuickBiz`).
fn file() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA").or_else(|| std::env::var_os("HOME"))?;
    let mut p = PathBuf::from(base);
    p.push("QuickBiz");
    p.push("session.json");
    Some(p)
}

pub fn load() -> Option<Saved> {
    let text = std::fs::read_to_string(file()?).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save(s: &Saved) {
    let Some(path) = file() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string(s) {
        let _ = std::fs::write(path, text);
    }
}

pub fn clear() {
    if let Some(p) = file() {
        let _ = std::fs::remove_file(p);
    }
}
