//! Sign in and create account. A blue brand panel on the left, the form on the right,
//! with the same rules as the mobile app (strong password, confirm password, currency, reset link).

use crate::app::{App, Msg};
use crate::model::CURRENCIES;
use crate::theme::*;
use crate::ui;
use eframe::egui::{self, CornerRadius, Margin, RichText, Sense};

pub struct LoginState {
    pub name: String,
    pub email: String,
    pub password: String,
    pub confirm: String,
    pub currency: String,
    pub show_password: bool,
    pub remember: bool,
    pub register: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub info: Option<String>,
}

impl Default for LoginState {
    fn default() -> Self {
        LoginState {
            name: String::new(),
            email: String::new(),
            password: String::new(),
            confirm: String::new(),
            currency: "BDT".into(),
            show_password: false,
            remember: true,
            register: false,
            busy: false,
            error: None,
            info: None,
        }
    }
}

pub const PASSWORD_HINT: &str = "At least 8 characters with a lowercase letter, an uppercase letter, a number and a symbol.";

/// Same rule as the server and the mobile app.
pub fn password_problem(pw: &str) -> Option<&'static str> {
    if pw.chars().count() < 8 {
        return Some("Password must be at least 8 characters.");
    }
    let lower = pw.chars().any(|c| c.is_lowercase());
    let upper = pw.chars().any(|c| c.is_uppercase());
    let digit = pw.chars().any(|c| c.is_ascii_digit());
    let symbol = pw.chars().any(|c| !c.is_alphanumeric());
    if lower && upper && digit && symbol { None } else { Some(PASSWORD_HINT) }
}

pub fn show(app: &mut App, ctx: &egui::Context) {
    let wide = ctx.screen_rect().width() > 940.0;
    if wide {
        egui::SidePanel::left("brand")
            .exact_width((ctx.screen_rect().width() * 0.44).clamp(380.0, 560.0))
            .resizable(false)
            .frame(egui::Frame::new().fill(BRAND).inner_margin(Margin::same(48)))
            .show(ctx, |ui| brand(ui, ctx));
    }
    egui::CentralPanel::default().frame(egui::Frame::new().fill(SURFACE)).show(ctx, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                let top = ((ui.available_height() - 560.0) / 2.0).max(28.0);
                ui.add_space(top);
                if !wide {
                    if let Some(tex) = crate::logo::texture(ctx) {
                        ui.image((tex.id(), egui::vec2(64.0, 64.0)));
                    }
                    ui.label(RichText::new("QuickBiz").font(bold_font(26.0)).color(BRAND));
                    ui.add_space(14.0);
                }
                ui.allocate_ui_with_layout(egui::vec2(420.0, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_width(420.0);
                    form(app, ctx, ui);
                });
                ui.add_space(24.0);
            });
        });
    });
}

fn brand(ui: &mut egui::Ui, ctx: &egui::Context) {
    let rect = ui.max_rect().expand(48.0);
    let p = ui.painter().with_clip_rect(rect);
    p.circle_filled(egui::pos2(rect.right() - 30.0, rect.top() + 40.0), 190.0, tint_on(BRAND, 0.08));
    p.circle_filled(egui::pos2(rect.left() + 20.0, rect.bottom() - 30.0), 130.0, tint_on(BRAND, 0.06));
    ui.horizontal(|ui| {
        if let Some(tex) = crate::logo::texture(ctx) {
            egui::Frame::new().fill(WHITE).corner_radius(CornerRadius::same(14)).inner_margin(Margin::same(4)).show(ui, |ui| {
                ui.image((tex.id(), egui::vec2(40.0, 40.0)));
            });
        }
        ui.label(RichText::new("QuickBiz").font(bold_font(26.0)).color(WHITE));
    });
    ui.add_space((ui.available_height() * 0.22).max(24.0));
    ui.label(RichText::new("Your daily accounts,").font(bold_font(38.0)).color(WHITE));
    ui.label(RichText::new("made simple.").font(bold_font(38.0)).color(WHITE));
    ui.add_space(10.0);
    ui.label(RichText::new("Income, expenses, stock and your team in one place. Everything stays in sync with the QuickBiz phone app.").color(tint_on(BRAND, 0.8)).size(15.5));
    ui.add_space(28.0);
    for (glyph, title, sub) in [
        (ic::CARD, "Track every taka", "Sales, expenses and money customers still owe"),
        (ic::BOX, "Know your stock", "Low-stock alerts and full stock history"),
        (ic::PEOPLE, "Work as a team", "Invite staff and choose what they can see"),
    ] {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(egui::vec2(42.0, 42.0), Sense::hover());
            ui.painter().rect_filled(r, CornerRadius::same(12), tint_on(BRAND, 0.16));
            ui::paint_icon(ui, r, glyph, 18.0, WHITE);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(RichText::new(title).font(bold_font(15.0)).color(WHITE));
                ui.label(RichText::new(sub).color(tint_on(BRAND, 0.75)).size(13.0));
            });
        });
        ui.add_space(12.0);
    }
}

fn password_field(ui: &mut egui::Ui, label: &str, value: &mut String, hint: &str, show: &mut bool) -> egui::Response {
    ui.label(RichText::new(label).font(bold_font(13.0)).color(MUTED));
    ui.horizontal(|ui| {
        let w = ui.available_width() - 44.0;
        let r = ui.add(egui::TextEdit::singleline(value).password(!*show).hint_text(hint).desired_width(w).margin(Margin::symmetric(10, 8)));
        let glyph = if *show { ic::EYE_OFF } else { ic::EYE };
        let b = egui::Button::new(ui::icon_text(glyph, 14.0, MUTED)).fill(WHITE).stroke(egui::Stroke::new(1.0_f32, LINE_STRONG)).corner_radius(CornerRadius::same(9)).min_size(egui::vec2(36.0, 34.0));
        if ui.add(b).on_hover_text(if *show { "Hide password" } else { "Show password" }).clicked() {
            *show = !*show;
        }
        r
    })
    .inner
}

fn form(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let mut submit = false;
    let mut reset = false;
    ui::card(ui, |ui| {
        let l = &mut app.login;
        ui::icon_box(ui, if l.register { ic::PERSON } else { ic::LOCK }, BRAND, 46.0);
        ui.add_space(6.0);
        ui.label(RichText::new(if l.register { "Create your account" } else { "Welcome back" }).font(bold_font(24.0)));
        ui.label(RichText::new(if l.register { "Start tracking your business in a minute." } else { "Sign in to QuickBiz and pick up where you left off." }).color(MUTED));
        ui.add_space(12.0);
        if let Some(i) = &l.info {
            crate::app::banner(ui, ic::CHECK_CIRCLE, i, GREEN, GREEN_SOFT);
        }
        if l.register {
            ui::field(ui, "Full name", &mut l.name, "John Doe");
            ui.add_space(4.0);
        }
        ui::field(ui, "Email address", &mut l.email, "name@company.com");
        ui.add_space(4.0);
        let hint = if l.register { "8+ chars: aA, 1 and a symbol" } else { "Enter your password" };
        let pw = password_field(ui, "Password", &mut l.password, hint, &mut l.show_password);
        let mut enter = pw.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if l.register {
            ui.add_space(4.0);
            let c = password_field(ui, "Confirm password", &mut l.confirm, "Repeat your password", &mut l.show_password);
            enter |= c.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.add_space(8.0);
            ui.label(RichText::new("DEFAULT CURRENCY").font(bold_font(11.5)).color(MUTED).extra_letter_spacing(1.0));
            ui.horizontal_wrapped(|ui| {
                for (code, sym) in CURRENCIES {
                    if ui::chip(ui, &format!("{sym} {code}"), l.currency == code).clicked() {
                        l.currency = code.to_string();
                    }
                }
            });
        } else {
            ui.horizontal(|ui| {
                ui.checkbox(&mut l.remember, "Keep me signed in");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui::link(ui, "Forgot password?").clicked() {
                        reset = true;
                    }
                });
            });
        }
        if let Some(e) = &l.error {
            ui.add_space(6.0);
            crate::app::banner(ui, ic::WARNING, e, RED, RED_SOFT);
        }
        ui.add_space(10.0);
        let label = if l.busy { "Please wait…" } else if l.register { "Create account" } else { "Sign in" };
        let go = ui.add_enabled(!l.busy, ui::primary_button(label).min_size(egui::vec2(ui.available_width(), 44.0))).clicked();
        if (go || enter) && !l.busy {
            submit = true;
        }
        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                let (q, a) = if l.register { ("Already have an account?", "Sign in") } else { ("Don't have an account?", "Create one") };
                ui.label(RichText::new(q).color(MUTED));
                if ui::link(ui, a).clicked() {
                    l.register = !l.register;
                    l.error = None;
                    l.info = None;
                    l.confirm.clear();
                }
            });
        });
    });
    if reset {
        request_reset(app, ctx);
    } else if submit {
        submit_form(app, ctx);
    }
}

fn request_reset(app: &mut App, ctx: &egui::Context) {
    let l = &mut app.login;
    let email = l.email.trim().to_string();
    l.info = None;
    if email.is_empty() || !email.contains('@') {
        l.error = Some("Enter your email address above first, then click \"Forgot password?\" again.".into());
        return;
    }
    l.error = None;
    l.busy = true;
    app.spawn(ctx, move |c, tx| {
        let _ = tx.send(Msg::ResetSent(c.reset_password(&email)));
    });
}

fn submit_form(app: &mut App, ctx: &egui::Context) {
    let l = &mut app.login;
    let email = l.email.trim().to_string();
    let password = l.password.clone();
    l.error = None;
    l.info = None;
    if l.register && l.name.trim().is_empty() {
        l.error = Some("Enter your name.".into());
        return;
    }
    if email.is_empty() || !email.contains('@') {
        l.error = Some("Enter a valid email address.".into());
        return;
    }
    if l.register {
        if let Some(p) = password_problem(&password) {
            l.error = Some(p.into());
            return;
        }
        if password != l.confirm {
            l.error = Some("The two passwords do not match.".into());
            return;
        }
    } else if password.is_empty() {
        l.error = Some("Enter your password.".into());
        return;
    }
    l.busy = true;
    let (register, name, remember, currency) = (l.register, l.name.trim().to_string(), l.remember, l.currency.clone());
    app.client.set_remember(remember);
    app.spawn(ctx, move |c, tx| {
        if register {
            let _ = tx.send(Msg::SignedUp(c.sign_up(&name, &email, &password, &currency)));
        } else {
            let _ = tx.send(Msg::SignedIn(c.sign_in(&email, &password)));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_rules_match_the_phone() {
        assert!(password_problem("short").is_some());
        assert!(password_problem("alllowercase1!").is_some());
        assert!(password_problem("NoSymbol123").is_some());
        assert!(password_problem("Good1Pass!").is_none());
    }
}
