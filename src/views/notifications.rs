//! Notification settings. They are saved to your account and decide which push alerts
//! the QuickBiz phone app shows; the Windows app itself does not send push alerts.

use crate::app::{App, Tab};
use crate::model::NotificationSettings;
use crate::theme::*;
use crate::ui;
use eframe::egui::{self, RichText};
use serde_json::json;

pub fn show(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    if ui::back_link(ui, "Back to Profile") {
        app.go(Tab::Profile, ctx);
    }
    let mut s: NotificationSettings = app.profile.notification_settings.unwrap_or_default();
    let before = s;
    ui.allocate_ui_with_layout(egui::vec2(ui.available_width().min(680.0), 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
        crate::app::banner(ui, ic::INFO, "These settings are saved to your account and control the alerts on your phone.", BLUE, BLUE_SOFT);
        ui::card(ui, |ui| {
            let rows: [(&str, &str, &str, egui::Color32, &mut bool); 3] = [
                (ic::WARNING, "Low stock alerts", "When a product reaches its low-stock level", RED, &mut s.low_stock),
                (ic::CARD, "New sales", "When someone on your team records a sale", GREEN, &mut s.new_sale),
                (ic::BELL, "News and updates", "New features and announcements from QuickBiz", BRAND, &mut s.announcements),
            ];
            for (i, (glyph, title, sub, color, value)) in rows.into_iter().enumerate() {
                if i > 0 {
                    ui::divider(ui);
                }
                ui.horizontal(|ui| {
                    ui.set_min_height(64.0);
                    ui::icon_box(ui, glyph, color, 40.0);
                    ui.vertical(|ui| {
                        ui.add_space(12.0);
                        ui.spacing_mut().item_spacing.y = 1.0;
                        ui.label(RichText::new(title).font(bold_font(14.5)));
                        ui.label(RichText::new(sub).color(MUTED).size(12.5));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui::toggle(ui, value);
                    });
                });
            }
        });
    });
    if s != before {
        app.profile.notification_settings = Some(s);
        let uid = app.profile.uid.clone();
        app.write(ctx, "Notification settings saved.", move |c| c.update_me(&uid, json!({ "notificationSettings": s })));
    }
}
