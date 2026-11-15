use crate::app::{card, primary_button, App, Msg};
use crate::theme::*;
use eframe::egui::{self, RichText};

#[derive(Default)]
pub struct LoginState {
    pub name: String,
    pub email: String,
    pub password: String,
    pub remember: bool,
    pub register: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub info: Option<String>,
}

pub fn show(app: &mut App, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() * 0.12).max(10.0));
            if let Some(tex) = crate::logo::texture(ctx) {
                ui.image((tex.id(), egui::vec2(72.0, 72.0)));
            }
            ui.add_space(6.0);
            ui.label(RichText::new("QuickBiz").font(bold_font(30.0)).color(BRAND));
            ui.label(RichText::new("Income, expenses and stock in one place").color(MUTED));
            ui.add_space(18.0);
            ui.allocate_ui(egui::vec2(380.0, 0.0), |ui| {
                card(ui, |ui| {
                    let l = &mut app.login;
                    ui.label(RichText::new(if l.register { "Create your account" } else { "Sign in" }).font(bold_font(18.0)));
                    ui.add_space(6.0);
                    if let Some(i) = &l.info {
                        ui.label(RichText::new(i).color(GREEN));
                    }
                    if l.register {
                        ui.label("Your name");
                        ui.add(egui::TextEdit::singleline(&mut l.name).desired_width(f32::INFINITY));
                    }
                    ui.label("Email");
                    ui.add(egui::TextEdit::singleline(&mut l.email).desired_width(f32::INFINITY));
                    ui.label("Password");
                    let pw = ui.add(egui::TextEdit::singleline(&mut l.password).password(true).desired_width(f32::INFINITY));
                    if !l.register {
                        ui.checkbox(&mut l.remember, "Keep me signed in");
                    }
                    if let Some(e) = &l.error {
                        ui.label(RichText::new(e).color(RED));
                    }
                    ui.add_space(6.0);
                    let enter = pw.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let label = if l.register { "Create account" } else { "Sign in" };
                    let go = ui.add_enabled(!l.busy, primary_button(label).min_size(egui::vec2(ui.available_width(), 38.0))).clicked();
                    if (go || enter) && !l.busy {
                        submit(app, ctx);
                    }
                    let l = &mut app.login;
                    ui.add_space(4.0);
                    let toggle = if l.register { "I already have an account" } else { "Create an account" };
                    if ui.link(toggle).clicked() {
                        l.register = !l.register;
                        l.error = None;
                        l.info = None;
                    }
                    if l.busy {
                        ui.spinner();
                    }
                });
            });
        });
    });
}

fn submit(app: &mut App, ctx: &egui::Context) {
    let l = &mut app.login;
    let email = l.email.trim().to_string();
    let password = l.password.clone();
    l.error = None;
    l.info = None;
    if email.is_empty() || !email.contains('@') {
        l.error = Some("Enter a valid email address.".into());
        return;
    }
    if password.len() < 6 {
        l.error = Some("Password must be at least 6 characters.".into());
        return;
    }
    l.busy = true;
    let (register, name, remember) = (l.register, l.name.trim().to_string(), l.remember);
    app.client.set_remember(remember);
    app.spawn(ctx, move |c, tx| {
        if register {
            let _ = tx.send(Msg::SignedUp(c.sign_up(&name, &email, &password)));
        } else {
            let _ = tx.send(Msg::SignedIn(c.sign_in(&email, &password)));
        }
    });
}
