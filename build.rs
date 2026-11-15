// Embeds the app icon and version info into the Windows .exe.
fn main() {
    println!("cargo:rerun-if-env-changed=QB_BUILD");
    let build = std::env::var("QB_BUILD")
        .ok()
        .filter(|b| !b.is_empty() && b.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or_else(|| "0".into());
    println!("cargo:rustc-env=QB_BUILD_NUMBER={build}");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/images/icon.ico");
        res.set("ProductName", "QuickBiz");
        res.set("FileDescription", "QuickBiz");
        if let Err(e) = res.compile() {
            println!("cargo:warning=could not embed icon: {e}");
        }
    }
}
