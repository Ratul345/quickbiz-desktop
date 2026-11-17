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
    pub business_category: Option<String>,
    #[serde(rename = "businessLogoURL")]
    pub business_logo_url: Option<String>,
    #[serde(rename = "photoURL")]
    pub photo_url: Option<String>,
    pub currency_code: Option<String>,
    pub system_role: Option<String>,
    pub tier: Option<String>,
    pub is_pro: Option<bool>,
    pub is_verified: Option<bool>,
    pub deletion_scheduled_at: Option<String>,
    pub permissions: Option<Permissions>,
    pub notification_settings: Option<NotificationSettings>,
}

/// What a business owner lets an employee see and do (same keys as the mobile app).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Permissions {
    pub can_view_price: bool,
    pub can_view_stock: bool,
    pub can_manage_inventory: bool,
    pub can_manage_transactions: bool,
    pub can_view_reports: bool,
}

impl Default for Permissions {
    /// Defaults for a new employee, as in the mobile app.
    fn default() -> Self {
        Permissions { can_view_price: true, can_view_stock: true, can_manage_inventory: false, can_manage_transactions: false, can_view_reports: false }
    }
}

/// Which push notifications the person wants on their phone.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct NotificationSettings {
    pub low_stock: bool,
    pub new_sale: bool,
    pub announcements: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        NotificationSettings { low_stock: true, new_sale: true, announcements: true }
    }
}

pub const BUSINESS_CATEGORIES: [&str; 5] = ["General Retail", "Mobile Repair", "Restaurant", "Services", "Other"];

/// Quick category buttons as (label, value saved), the same lists as the mobile app.
pub fn category_presets(business: bool, business_category: &str, income: bool) -> Vec<(&'static str, &'static str)> {
    let all: &[(&str, &str, bool)] = if !business {
        &[
            ("Food & Dining", "Food", false), ("Groceries", "Groceries", false), ("Shopping", "Shopping", false),
            ("Transport", "Transport", false), ("Bills & Utilities", "Bills", false), ("Entertainment", "Entertainment", false),
            ("Health", "Health", false), ("Salary", "Salary", true), ("Gift", "Gift", true),
            ("Other Income", "Other Income", true), ("Other Expense", "Other Expense", false),
        ]
    } else {
        match business_category {
            "General Retail" => &[
                ("Sales Revenue", "Sales", true), ("Stock Purchase", "Stock Purchase", false), ("Shop Rent", "Rent", false),
                ("Electricity/Water", "Utilities", false), ("Staff Salary", "Salaries", false), ("Marketing/Ads", "Marketing", false),
                ("Packaging", "Supplies", false), ("Other Income", "Other Income", true), ("Other Expense", "Other Expense", false),
            ],
            "Mobile Repair" => &[
                ("Service Charge", "Repair Service", true), ("Parts Sale", "Parts Sale", true), ("Parts Purchase", "Stock Purchase", false),
                ("Accessories", "Sales", true), ("Shop Rent", "Rent", false), ("Electricity", "Utilities", false),
                ("Tools/Equip", "Equipment", false), ("Other Income", "Other Income", true), ("Other Expense", "Other Expense", false),
            ],
            "Restaurant" => &[
                ("Food Sales", "Sales", true), ("Catering", "Catering", true), ("Raw Materials", "Stock Purchase", false),
                ("Kitchen Equip", "Equipment", false), ("Chef/Waiters", "Salaries", false), ("Gas/Utility", "Utilities", false),
                ("Rent", "Rent", false), ("Other Income", "Other Income", true), ("Other Expense", "Other Expense", false),
            ],
            "Services" => &[
                ("Service Fee", "Sales", true), ("Consultancy", "Consultancy", true), ("Software/Sub", "Expenses", false),
                ("Office Rent", "Rent", false), ("Utilities", "Utilities", false), ("Staff Payroll", "Salaries", false),
                ("Marketing", "Marketing", false), ("Other Income", "Other Income", true), ("Other Expense", "Other Expense", false),
            ],
            _ => &[
                ("Sales/Income", "Sales", true), ("Expenses", "Expenses", false), ("Rent", "Rent", false),
                ("Utilities", "Utilities", false), ("Salaries", "Salaries", false), ("Stock/Supplies", "Stock Purchase", false),
                ("Other Income", "Other Income", true), ("Other Expense", "Other Expense", false),
            ],
        }
    };
    all.iter().filter(|c| c.2 == income).map(|c| (c.0, c.1)).collect()
}

/// A team member as the owner sees them.
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Employee {
    pub uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    #[serde(rename = "photoURL")]
    pub photo_url: Option<String>,
    pub permissions: Option<Permissions>,
}

impl Employee {
    pub fn name(&self) -> String {
        self.display_name.clone().filter(|n| !n.trim().is_empty()).or_else(|| self.email.clone()).unwrap_or_else(|| "Employee".into())
    }
}

/// Result of looking someone up by email before inviting them.
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct FoundUser {
    pub uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub business_owner_id: Option<String>,
}

/// An invitation to join someone's business.
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct TeamInvite {
    pub id: String,
    pub owner_name: Option<String>,
    pub business_name: Option<String>,
}

/// A message to QuickBiz support and its reply.
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Ticket {
    pub id: String,
    pub subject: String,
    pub message: String,
    pub reply: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
}

impl Profile {
    /// Data belongs to the business owner, so employees read and write under the owner's id.
    pub fn data_owner(&self) -> String {
        match self.business_owner_id.as_deref() {
            Some(o) if !o.is_empty() => o.to_string(),
            _ => self.uid.clone(),
        }
    }
    pub fn is_employee(&self) -> bool {
        self.role.as_deref() == Some("employee")
    }
    pub fn is_owner(&self) -> bool {
        self.role.as_deref() == Some("owner")
    }
    pub fn has_business(&self) -> bool {
        self.is_owner() || self.is_employee()
    }
    /// What this person may do in business mode. Owners may do everything.
    pub fn perms(&self) -> Permissions {
        if self.is_employee() {
            self.permissions.unwrap_or_default()
        } else {
            Permissions { can_view_price: true, can_view_stock: true, can_manage_inventory: true, can_manage_transactions: true, can_view_reports: true }
        }
    }
    pub fn first_name(&self) -> String {
        let n = self.name();
        let n = n.split('@').next().unwrap_or("").to_string();
        n.split_whitespace().next().unwrap_or("").to_string()
    }
    pub fn initial(&self) -> String {
        self.name().chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| "U".into())
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
    pub created_by: Option<String>,
    pub currency_code: Option<String>,
    pub description: Option<String>,
    pub pickup_time: Option<String>,
}

impl Transaction {
    pub fn kind(&self) -> Kind {
        Kind::of(self.sub_type.as_deref(), self.customer_name.as_deref())
    }
    pub fn is_income(&self) -> bool {
        self.kind == "income"
    }
    pub fn day(&self) -> Option<chrono::NaiveDate> {
        chrono::DateTime::parse_from_rfc3339(&self.date)
            .ok()
            .map(|d| d.with_timezone(&chrono::Local).date_naive())
    }
}

/// Extra sale types per business category. They reuse existing columns, so the phone app
/// (src/lib/businessKinds.ts) reads and writes them the same way:
///   repair  customer, phone, problem, total/paid/due, pickup time, status
///   credit  customer, phone, total/paid/due, status
///   job     client (customer), phone, work (problem), total/paid/due, due date (pickup time), status
///   order   order type (description), table (problem), customer, phone, dishes (items); paid in full
/// To add a new kind later, add it here and in the phone app.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Sale,
    Repair,
    Credit,
    Job,
    Order,
}

pub struct Special {
    pub kind: Kind,
    pub label: &'static str,
    pub sale_label: &'static str,
}

pub const ORDER_TYPES: [&str; 3] = ["Dine-in", "Takeaway", "Delivery"];

impl Kind {
    pub fn of(sub_type: Option<&str>, customer: Option<&str>) -> Kind {
        match sub_type {
            Some("repair") => Kind::Repair,
            Some("credit") => Kind::Credit,
            Some("job") => Kind::Job,
            Some("order") => Kind::Order,
            None | Some("") if customer.map(|c| !c.is_empty()).unwrap_or(false) => Kind::Repair, // older repair entries
            _ => Kind::Sale,
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Kind::Sale => "sale",
            Kind::Repair => "repair",
            Kind::Credit => "credit",
            Kind::Job => "job",
            Kind::Order => "order",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Kind::Sale => "Sale",
            Kind::Repair => "Repair",
            Kind::Credit => "Credit sale",
            Kind::Job => "Job",
            Kind::Order => "Order",
        }
    }
    /// Customer pays part now and may owe the rest.
    pub fn has_due(self) -> bool {
        matches!(self, Kind::Repair | Kind::Credit | Kind::Job)
    }
}

/// The special kind each business category gets.
pub fn special_for(business_category: Option<&str>) -> Special {
    match business_category.unwrap_or("Other") {
        "Mobile Repair" => Special { kind: Kind::Repair, label: "Repair job", sale_label: "Sale" },
        "General Retail" => Special { kind: Kind::Credit, label: "Credit sale", sale_label: "Cash sale" },
        "Restaurant" => Special { kind: Kind::Order, label: "Order", sale_label: "Quick sale" },
        "Services" => Special { kind: Kind::Job, label: "Job", sale_label: "Payment" },
        _ => Special { kind: Kind::Credit, label: "Customer owes", sale_label: "Sale" },
    }
}

/// A later payment on an entry with money still due: (amount, paidAmount, dueAmount, new status).
pub fn apply_payment(t: &Transaction, payment: f64) -> Result<(f64, f64, f64, Option<&'static str>), &'static str> {
    let due = t.due_amount.unwrap_or(0.0);
    if !(payment > 0.0) {
        return Err("Enter an amount greater than 0.");
    }
    if payment > due + 0.0001 {
        return Err("That is more than the amount still due.");
    }
    let paid = t.paid_amount.unwrap_or(t.amount) + payment;
    let left = ((due - payment) * 100.0).round().max(0.0) / 100.0;
    let status = if left == 0.0 && t.sub_type.as_deref() == Some("credit") { Some("completed") } else { None };
    Ok((t.amount + payment, paid, left, status))
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
    pub description: Option<String>,
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

/// The public bits of the app settings row: the maintenance flag and the live *Windows* release.
/// (The Android release fields in the same row belong to the phone app and are never used here.)
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub maintenance_mode: Option<bool>,
    pub windows_version: Option<String>,
}

/// The columns the app may read. The settings table is readable column by column only,
/// so asking for `*` is refused by the server.
pub const SETTINGS_COLUMNS: &str = "maintenanceMode,windowsVersion";

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
    fn settings_read_windows_fields_only() {
        let s: Settings = serde_json::from_str(r#"{"maintenanceMode":true,"windowsVersion":"1.0.0","latestVersion":"1.5.1"}"#).unwrap();
        assert_eq!(s.maintenance_mode, Some(true));
        assert_eq!(s.windows_version.as_deref(), Some("1.0.0"));
        assert!(!SETTINGS_COLUMNS.contains('*') && !SETTINGS_COLUMNS.contains("latestVersion"));
    }

    #[test]
    fn kinds_and_payments() {
        assert_eq!(Kind::of(None, Some("Rahim")), Kind::Repair);
        assert_eq!(Kind::of(Some("credit"), Some("Rahim")), Kind::Credit);
        assert_eq!(Kind::of(Some("sale"), None), Kind::Sale);
        assert_eq!(special_for(Some("Restaurant")).kind, Kind::Order);
        assert_eq!(special_for(None).kind, Kind::Credit);
        let t = Transaction { amount: 200.0, paid_amount: Some(200.0), due_amount: Some(300.0), sub_type: Some("credit".into()), ..Default::default() };
        assert_eq!(apply_payment(&t, 300.0), Ok((500.0, 500.0, 0.0, Some("completed"))));
        assert_eq!(apply_payment(&t, 100.0), Ok((300.0, 300.0, 200.0, None)));
        assert!(apply_payment(&t, 400.0).is_err());
        assert!(apply_payment(&t, 0.0).is_err());
    }

    #[test]
    fn employee_data_belongs_to_owner() {
        let p = Profile { uid: "emp".into(), business_owner_id: Some("boss".into()), ..Default::default() };
        assert_eq!(p.data_owner(), "boss");
        let o = Profile { uid: "me".into(), ..Default::default() };
        assert_eq!(o.data_owner(), "me");
    }
}
