//! Public configuration. The publishable key is designed to be public: what it can
//! do is limited by Row Level Security on the server. No secret ever lives in this app.

pub const SUPABASE_URL: &str = "https://mbdcyvgwoikbdrsdcflq.supabase.co";
pub const SUPABASE_KEY: &str = "sb_publishable_gdZm_EWXsNuwV4V12nKexQ_mmREMKCN";
pub const WEB_URL: &str = "https://quickbiz.vercel.app";
pub const PRIVACY_URL: &str = "https://quickbiz.vercel.app/privacy";
/// Opens the QuickBiz page in the Microsoft Store app.
pub const STORE_URL: &str = "ms-windows-store://pdp/?ProductId=9PP755N6HGN4";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// CI build number baked in at compile time (0 for local builds).
#[allow(dead_code)]
pub fn build_number() -> u64 {
    env!("QB_BUILD_NUMBER").parse().unwrap_or(0)
}

/// True in the Microsoft Store build, where the Store (not the app) installs updates.
pub const STORE_BUILD: bool = cfg!(feature = "store");
