//! Rows as the QuickBiz database stores them (camelCase column names, same as the mobile app).

use serde::{Deserialize, Serialize};

pub const CURRENCIES: [(&str, &str); 7] = [
    ("BDT", "৳"),
    ("USD", "$"),
    ("EUR", "€"),
    ("GBP", "£"),
    ("INR", "₹"),
    ("AUD", "A$"),
    ("CAD", "C$"),
];

pub fn currency_symbol(code: &str) -> &'static str {
    CURRENCIES.iter().find(|(c, _)| *c == code).map(|(_, s)| *s).unwrap_or("৳")
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub business_owner_id: Option<String>,
    pub business_id: Option<String>,
    pub business_name: Option<String>,
    pub currency_code: Option<String>,
}

impl Profile {
    /// Data belongs to the business owner, so employees read and write under the owner's id.
    pub fn data_owner(&self) -> String {
        match self.business_owner_id.as_deref() {
            Some(o) if !o.is_empty() => o.to_string(),
            _ => self.uid.clone(),
        }
    }
    pub fn currency(&self) -> String {
        self.currency_code.clone().filter(|c| !c.is_empty()).unwrap_or_else(|| "BDT".into())
    }
    pub fn name(&self) -> String {
        self.display_name
            .clone()
            .filter(|n| !n.trim().is_empty())
            .or_else(|| self.email.clone())
            .unwrap_or_else(|| "You".into())
    }
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Transaction {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub amount: f64,
    pub category: Option<String>,
    pub note: Option<String>,
    pub date: String,
    pub created_by_name: Option<String>,
    pub product_name: Option<String>,
    pub quantity: Option<f64>,
    pub sub_type: Option<String>,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub problem: Option<String>,
    pub total_amount: Option<f64>,
    pub paid_amount: Option<f64>,
    pub due_amount: Option<f64>,
    pub status: Option<String>,
}

impl Transaction {
    pub fn is_income(&self) -> bool {
        self.kind == "income"
    }
    pub fn day(&self) -> Option<chrono::NaiveDate> {
        chrono::DateTime::parse_from_rfc3339(&self.date)
            .ok()
            .map(|d| d.with_timezone(&chrono::Local).date_naive())
    }
}

/// What the "add transaction" form sends to the `add_transaction` database function.
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct NewTransaction {
    #[serde(rename = "type")]
    pub kind: String,
    pub amount: f64,
    pub category: String,
    pub note: String,
    pub currency_code: String,
    pub currency_symbol: String,
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pickup_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    pub items: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub sku: Option<String>,
    pub price: f64,
    pub cost: f64,
    pub stock_quantity: f64,
    pub min_stock_level: f64,
    pub category: Option<String>,
}

impl Product {
    pub fn is_low(&self) -> bool {
        self.stock_quantity <= self.min_stock_level
    }
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct NewProduct {
    pub name: String,
    pub sku: String,
    pub price: f64,
    pub cost: f64,
    pub stock_quantity: f64,
    pub min_stock_level: f64,
    pub category: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct StockMovement {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub quantity: f64,
    pub note: Option<String>,
    pub date: String,
    pub created_by_name: Option<String>,
}

/// Public app settings row (maintenance flag, latest version, download link, notes).
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub maintenance_mode: Option<bool>,
    pub latest_version: Option<String>,
    pub download_url: Option<String>,
    pub changelog: Option<String>,
    pub force_update: Option<bool>,
}

/// "1.10.0" is newer than "1.9.0" (compared number by number, like the mobile app).
pub fn is_newer_version(latest: &str, current: &str) -> bool {
    fn parts(v: &str) -> Vec<u64> {
        v.trim()
            .trim_start_matches(['v', 'V'])
            .split(['-', '+'])
            .next()
            .unwrap_or("")
            .split('.')
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    }
    if latest.trim().is_empty() {
        return false;
    }
    let (a, b) = (parts(latest), parts(current));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if x != y {
            return x > y;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_numerically() {
        assert!(is_newer_version("1.10.0", "1.9.0"));
        assert!(!is_newer_version("1.5.1", "1.5.1"));
        assert!(!is_newer_version("", "1.0.0"));
        assert!(is_newer_version("v2.0", "1.9.9"));
    }

    #[test]
    fn employee_data_belongs_to_owner() {
        let p = Profile { uid: "emp".into(), business_owner_id: Some("boss".into()), ..Default::default() };
        assert_eq!(p.data_owner(), "boss");
        let o = Profile { uid: "me".into(), ..Default::default() };
        assert_eq!(o.data_owner(), "me");
    }
}
