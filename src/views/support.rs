//! Help & Support: send a question, problem, feature idea or feedback to the QuickBiz team
//! and read the replies. Uses the same support inbox as the mobile app and the admin app.

use crate::app::{App, Tab};
use crate::config;
use crate::model::Ticket;
use crate::theme::*;
use crate::ui;
use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke};
use serde_json::json;

/// (key, tag stored in the subject, label, icon, intro, subject hint, message hint)
const TYPES: [(&str, &str, &str, &str, &str, &str, &str); 4] = [
    ("question", "Question", "Question", ic::HELP, "Have a question? Send us a message and we will help you out.", "What is your question about?", "Write your question here..."),
    ("problem", "Problem", "Problem", ic::WARNING, "Something not working? Tell us what happened and we will look into it.", "What went wrong?", "What did you do, and what did you expect to happen?"),
    ("feature", "Feature request", "Feature idea", ic::BULB, "Is there something QuickBiz should do that it does not yet? We read every idea.", "What feature would help you?", "Describe the feature and how you would use it in your work..."),
    ("feedback", "Feedback", "Feedback", ic::HEART, "Tell us what you like and what we can do better.", "A short title", "Share your thoughts about the app..."),
];

#[derive(Default)]
pub struct SupportState {
    pub kind: usize,
    pub tab: u8, // 0 new, 1 my tickets
    pub subject: String,
    pub message: String,
    pub replying: Option<String>,
    pub reply: String,
}

impl SupportState {
    pub fn start(&mut self, key: &str) {
        self.kind = TYPES.iter().position(|t| t.0 == key).unwrap_or(0);
        self.tab = 0;
    }
}

/// Splits "[Feedback] Nice app" into the type label and the title the person wrote.
fn split_subject(s: &str) -> (Option<usize>, String) {
    if let Some(rest) = s.strip_prefix('[') {
        if let Some((tag, title)) = rest.split_once(']') {
            if let Some(i) = TYPES.iter().position(|t| t.1 == tag) {
                let title = title.trim();
                return (Some(i), if title.is_empty() { TYPES[i].2.to_string() } else { title.to_string() });
            }
        }
    }
    (None, s.to_string())
}

pub fn show(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    if ui::back_link(ui, "Back to Profile") {
        app.go(Tab::Profile, ctx);
    }
    let n = app.tickets.len();
    let tickets_label = format!("My tickets ({n})");
    let mut tab = app.support.tab;
    if ui::segmented(ui, &mut tab, &[(0, "New message"), (1, tickets_label.as_str())]) {
        app.support.tab = tab;
        if tab == 1 {
            app.load_tickets(ctx);
        }
    }
    ui.add_space(12.0);
    ui.allocate_ui_with_layout(egui::vec2(ui.available_width().min(820.0), 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
        if app.support.tab == 0 {
            new_message(app, ctx, ui);
        } else {
            tickets(app, ctx, ui);
        }
    });
}

fn new_message(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let mut send = false;
    ui::card(ui, |ui| {
        ui.label(RichText::new("What is it about?").font(bold_font(14.5)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (i, t) in TYPES.iter().enumerate() {
                if ui::chip(ui, t.2, app.support.kind == i).clicked() {
                    app.support.kind = i;
                }
            }
        });
        let t = TYPES[app.support.kind];
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui::icon_box(ui, t.3, BRAND, 34.0);
            ui.label(RichText::new(t.4).color(MUTED));
        });
        ui.add_space(8.0);
        ui::field(ui, "Subject", &mut app.support.subject, t.5);
        ui.add_space(4.0);
        ui::multiline(ui, "Message", &mut app.support.message, t.6, 6);
        ui.add_space(12.0);
        if ui.add(ui::primary_button("Send message").min_size(egui::vec2(180.0, 42.0))).clicked() {
            send = true;
        }
    });
    if !send {
        return;
    }
    let s = &app.support;
    if s.subject.trim().is_empty() || s.message.trim().is_empty() {
        app.say("Please fill in both the subject and the message.", false);
        return;
    }
    let t = TYPES[s.kind];
    let email = app.session.as_ref().map(|x| x.email.clone()).unwrap_or_else(|| "No email provided".into());
    let row = json!({
        "userId": app.profile.uid,
        "userName": app.profile.display_name.clone().filter(|n| !n.is_empty()).unwrap_or_else(|| email.split('@').next().unwrap_or("User").to_string()),
        "userEmail": email,
        "businessName": app.profile.business_name.clone().unwrap_or_else(|| "No business".into()),
        "subject": format!("[{}] {}", t.1, s.subject.trim()),
        "message": s.message.trim(),
        "status": "pending",
        "platform": "windows",
        "appVersion": config::APP_VERSION,
    });
    let thanks = if t.0 == "feature" || t.0 == "feedback" {
        "Thank you! We read every message. Any reply appears in My tickets."
    } else {
        "Message sent. Our reply will appear in My tickets."
    };
    app.support.subject.clear();
    app.support.message.clear();
    app.support.tab = 1;
    app.write(ctx, thanks, move |c| c.send_ticket(row));
}

fn tickets(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let list: Vec<Ticket> = app.tickets.clone();
    if list.is_empty() {
        ui::card(ui, |ui| {
            ui::empty_state(ui, ic::MESSAGE, "No messages yet", "Messages you send to QuickBiz support appear here with our replies.");
        });
        return;
    }
    let mut send_reply: Option<(String, String)> = None;
    for t in &list {
        ui::card(ui, |ui| {
            let (kind, title) = split_subject(&t.subject);
            ui.horizontal(|ui| {
                ui::icon_box(ui, kind.map(|i| TYPES[i].3).unwrap_or(ic::MESSAGE), BRAND, 34.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(RichText::new(title).font(bold_font(15.0)));
                    let date = t
                        .created_at
                        .as_deref()
                        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
                        .map(|d| d.with_timezone(&chrono::Local).format("%d %b %Y, %H:%M").to_string())
                        .unwrap_or_default();
                    let label = kind.map(|i| TYPES[i].2).unwrap_or("Message");
                    ui.label(RichText::new(format!("{label}  ·  {date}")).color(MUTED).size(12.5));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    match t.status.as_deref() {
                        Some("replied") => ui::pill(ui, "Replied", GREEN, GREEN_SOFT),
                        Some("resolved") => ui::pill(ui, "Resolved", BRAND, BRAND_SOFT),
                        _ => ui::pill(ui, "Waiting for reply", AMBER, AMBER_SOFT),
                    };
                });
            });
            ui.add_space(8.0);
            ui.label(RichText::new(&t.message));
            if let Some(reply) = t.reply.as_ref().filter(|r| !r.trim().is_empty()) {
                ui.add_space(8.0);
                egui::Frame::new().fill(BRAND_FAINT).stroke(Stroke::new(1.0_f32, BRAND_SOFT)).corner_radius(CornerRadius::same(12)).inner_margin(Margin::same(12)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new("QuickBiz support").font(bold_font(13.0)).color(BRAND));
                    ui.label(RichText::new(reply));
                });
            }
            if t.status.as_deref() != Some("resolved") {
                ui.add_space(8.0);
                if app.support.replying.as_deref() == Some(t.id.as_str()) {
                    ui.add(egui::TextEdit::multiline(&mut app.support.reply).hint_text("Write your reply...").desired_rows(3).desired_width(f32::INFINITY).margin(Margin::symmetric(10, 8)));
                    ui.horizontal(|ui| {
                        if ui.add(ui::primary_button("Send reply")).clicked() && !app.support.reply.trim().is_empty() {
                            send_reply = Some((t.id.clone(), app.support.reply.trim().to_string()));
                        }
                        if ui.add(ui::secondary_button("Cancel")).clicked() {
                            app.support.replying = None;
                            app.support.reply.clear();
                        }
                    });
                } else if t.reply.as_ref().map(|r| !r.trim().is_empty()).unwrap_or(false) && ui::link(ui, "Reply").clicked() {
                    app.support.replying = Some(t.id.clone());
                    app.support.reply.clear();
                }
            }
        });
        ui.add_space(10.0);
    }
    if let Some((id, msg)) = send_reply {
        app.support.replying = None;
        app.support.reply.clear();
        app.write(ctx, "Your reply has been sent.", move |c| c.reply_ticket(&id, &msg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_type_tag_from_subject() {
        assert_eq!(split_subject("[Feedback] Nice app"), (Some(3), "Nice app".to_string()));
        assert_eq!(split_subject("[Feature request] Barcodes"), (Some(2), "Barcodes".to_string()));
        assert_eq!(split_subject("Plain subject"), (None, "Plain subject".to_string()));
        assert_eq!(split_subject("[Unknown] x"), (None, "[Unknown] x".to_string()));
    }
}
