#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod app;
mod config;
#[cfg(feature = "demo")]
mod demo;
mod logo;
mod model;
mod pictures;
mod store;
mod theme;
mod ui;
mod views;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("QuickBiz")
        .with_inner_size(if cfg!(feature = "demo") { [1366.0, 768.0] } else { [1180.0, 760.0] })
        .with_min_inner_size([900.0, 600.0]);
    if let Some(icon) = logo::window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("QuickBiz", options, Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
}
