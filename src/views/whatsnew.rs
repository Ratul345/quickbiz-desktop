//! "What's new" dialog: shown once after each update, and from Profile at any time.
//! The text is the latest section of WHATS_NEW.md, built into the app.

use crate::app::App;
use crate::config;
use crate::theme::*;
use crate::ui;
use eframe::egui::{self, RichText};

const NOTES: &str = include_str!("../../WHATS_NEW.md");

/// The newest section: everything before the first "---".
fn latest() -> &'static str {
    NOTES.split("\n---").next().unwrap_or(NOTES)
}

/// Shows the notes after an update (once per version), or when opened from Profile.
pub fn show(app: &mut App, ctx: &egui::Context) {
    if cfg!(feature = "demo") {
        return;
    }
    if app.prefs.last_seen_version != config::APP_VERSION && !app.whats_new {
        app.whats_new = true;
    }
    if !app.whats_new {
        return;
    }
    let mut done = false;
    let (_, closed) = ui::dialog(ctx, "whats-new", "What's new", 520.0, |ui| {
        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            for line in latest().lines() {
                let line = line.trim_end();
                if line.starts_with("# ") {
                    continue; // the dialog has its own title
                }
                let clean = line.replace("**", "");
                if let Some(h) = clean.strip_prefix("### ") {
                    ui.add_space(8.0);
                    ui.label(RichText::new(h).font(bold_font(16.0)));
                } else if let Some(b) = clean.strip_prefix("- ") {
                    ui.horizontal_top(|ui| {
                        ui.label(RichText::new("•").color(BRAND).font(bold_font(14.0)));
                        ui.add(egui::Label::new(RichText::new(b).color(INK)).wrap());
                    });
                } else if !clean.trim().is_empty() {
                    ui.add(egui::Label::new(RichText::new(clean).color(MUTED)).wrap());
                }
            }
        });
        ui.add_space(14.0);
        if ui.add(ui::primary_button("Got it").min_size(egui::vec2(ui.available_width(), 42.0))).clicked() {
            done = true;
        }
    });
    if done || closed {
        app.whats_new = false;
        app.prefs.last_seen_version = config::APP_VERSION.to_string();
        app.save_prefs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_section_is_this_version() {
        let v = env!("CARGO_PKG_VERSION");
        assert!(latest().contains(v), "WHATS_NEW.md must start with the notes for {v}");
        assert!(!latest().contains("previous version"));
    }
}
