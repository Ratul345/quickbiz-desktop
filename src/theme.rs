#![allow(dead_code)]
//! Design tokens shared with the QuickBiz mobile app (src/constants/theme.ts in quickbiz-expo).
//! Fonts are the ones Windows already ships (Segoe UI, Segoe MDL2 Assets), loaded at runtime
//! so the exe stays small. If they are missing the app falls back to egui's built-in font.

use eframe::egui::{self, epaint::Shadow, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const BRAND: Color32 = Color32::from_rgb(0x2D, 0x62, 0xED);
pub const BRAND_DARK: Color32 = Color32::from_rgb(0x1A, 0x43, 0xBF);
pub const BRAND_HOVER: Color32 = Color32::from_rgb(0x25, 0x54, 0xD4);
pub const BRAND_SOFT: Color32 = Color32::from_rgb(0xE3, 0xEB, 0xFF);
pub const BRAND_FAINT: Color32 = Color32::from_rgb(0xF0, 0xF4, 0xFF);
pub const INK: Color32 = Color32::from_rgb(0x0F, 0x17, 0x2A);
pub const MUTED: Color32 = Color32::from_rgb(0x64, 0x74, 0x8B);
pub const FAINT: Color32 = Color32::from_rgb(0x94, 0xA3, 0xB8);
pub const LINE: Color32 = Color32::from_rgb(0xE6, 0xEB, 0xF2);
pub const LINE_STRONG: Color32 = Color32::from_rgb(0xD5, 0xDC, 0xE8);
pub const SURFACE: Color32 = Color32::from_rgb(0xF8, 0xFA, 0xFC);
pub const SURFACE_MUTED: Color32 = Color32::from_rgb(0xF1, 0xF5, 0xF9);
pub const WHITE: Color32 = Color32::WHITE;
pub const GREEN: Color32 = Color32::from_rgb(0x0E, 0x9F, 0x6E);
pub const GREEN_SOFT: Color32 = Color32::from_rgb(0xE7, 0xF8, 0xF1);
pub const RED: Color32 = Color32::from_rgb(0xDC, 0x2F, 0x2F);
pub const RED_HOVER: Color32 = Color32::from_rgb(0xC2, 0x26, 0x26);
pub const RED_SOFT: Color32 = Color32::from_rgb(0xFD, 0xEC, 0xEC);
pub const AMBER: Color32 = Color32::from_rgb(0xB4, 0x74, 0x0B);
pub const AMBER_SOFT: Color32 = Color32::from_rgb(0xFE, 0xF4, 0xDC);

static ICONS_OK: AtomicBool = AtomicBool::new(false);

pub fn icons_available() -> bool {
    ICONS_OK.load(Ordering::Relaxed)
}

pub fn bold_family() -> FontFamily {
    FontFamily::Name("bold".into())
}

pub fn icon_family() -> FontFamily {
    FontFamily::Name("icons".into())
}

pub fn text_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn bold_font(size: f32) -> FontId {
    FontId::new(size, bold_family())
}

pub fn icon_font(size: f32) -> FontId {
    FontId::new(size, icon_family())
}

fn load_font(name: &str) -> Option<Vec<u8>> {
    let windir = std::env::var_os("WINDIR").map(std::path::PathBuf::from).unwrap_or_else(|| "C:\\Windows".into());
    std::fs::read(windir.join("Fonts").join(name)).ok()
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let default_prop = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();

    let mut regular = default_prop.clone();
    let mut bold = default_prop.clone();

    if let Some(bytes) = load_font("segoeui.ttf") {
        fonts.font_data.insert("segoe".into(), Arc::new(FontData::from_owned(bytes)));
        regular.insert(0, "segoe".into());
    }
    if let Some(bytes) = load_font("segoeuib.ttf").or_else(|| load_font("seguisb.ttf")) {
        fonts.font_data.insert("segoe-bold".into(), Arc::new(FontData::from_owned(bytes)));
        bold.insert(0, "segoe-bold".into());
    } else if regular.first().map(|n| n == "segoe").unwrap_or(false) {
        bold.insert(0, "segoe".into());
    }
    fonts.families.insert(FontFamily::Proportional, regular);
    fonts.families.insert(bold_family(), bold);

    // Segoe MDL2 Assets ships with Windows 10 and 11 and gives one consistent icon set.
    let icon_bytes = load_font("segmdl2.ttf");
    let have_icons = icon_bytes.is_some();
    if let Some(bytes) = icon_bytes {
        fonts.font_data.insert("mdl2".into(), Arc::new(FontData::from_owned(bytes)));
        fonts.families.insert(icon_family(), vec!["mdl2".into()]);
    } else {
        fonts.families.insert(icon_family(), default_prop);
    }
    ICONS_OK.store(have_icons, Ordering::Relaxed);
    ctx.set_fonts(fonts);
}

pub fn apply(ctx: &egui::Context) {
    install_fonts(ctx);

    let mut v = egui::Visuals::light();
    v.panel_fill = SURFACE;
    v.window_fill = WHITE;
    v.extreme_bg_color = WHITE;
    v.faint_bg_color = SURFACE_MUTED;
    v.override_text_color = Some(INK);
    v.selection.bg_fill = Color32::from_rgb(0xCF, 0xDC, 0xFF);
    v.selection.stroke = Stroke::new(1.5_f32, BRAND);
    v.hyperlink_color = BRAND;
    v.window_corner_radius = CornerRadius::same(16);
    v.menu_corner_radius = CornerRadius::same(10);
    v.window_stroke = Stroke::new(1.0_f32, LINE);
    v.window_shadow = Shadow { offset: [0, 12], blur: 32, spread: 0, color: Color32::from_black_alpha(34) };
    v.popup_shadow = Shadow { offset: [0, 6], blur: 18, spread: 0, color: Color32::from_black_alpha(28) };
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, LINE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, INK);
    v.widgets.inactive.bg_fill = SURFACE_MUTED;
    v.widgets.inactive.weak_bg_fill = WHITE;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, LINE_STRONG);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, BRAND);
    v.widgets.hovered.weak_bg_fill = BRAND_FAINT;
    v.widgets.hovered.bg_fill = BRAND_FAINT;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, BRAND);
    v.widgets.active.weak_bg_fill = BRAND_SOFT;
    v.widgets.active.bg_fill = BRAND_SOFT;
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(9);
        w.expansion = 0.0;
    }
    ctx.set_visuals(v);

    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(10.0, 8.0);
        s.spacing.button_padding = egui::vec2(14.0, 8.0);
        s.spacing.interact_size.y = 34.0;
        s.spacing.scroll = egui::style::ScrollStyle::floating();
        s.text_styles = [
            (TextStyle::Heading, bold_font(24.0)),
            (TextStyle::Body, text_font(14.5)),
            (TextStyle::Button, text_font(14.5)),
            (TextStyle::Small, text_font(12.5)),
            (TextStyle::Monospace, FontId::monospace(13.5)),
        ]
        .into();
    });
}
