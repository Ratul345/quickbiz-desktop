//! Profile: who you are, your business, app mode, account settings, support and legal links.
//! Same options as the Profile tab in the mobile app, arranged in two columns.

use crate::app::{open_url, App, Tab};
use crate::config;
use crate::model::*;
use crate::theme::*;
use crate::ui;
use eframe::egui::{self, CornerRadius, Margin, RichText, Sense, Stroke};
use serde_json::json;

#[derive(Clone, Copy, PartialEq)]
pub enum Confirm {
    SignOut,
    LeaveBusiness,
    DeleteBusiness,
    DeleteAccount,
    CancelDeletion,
}

#[derive(Default)]
pub struct ProfileState {
    pub edit: bool,
    pub name: String,
    pub currency: String,
    pub photo: Option<String>,
    pub business: bool,
    pub creating: bool,
    pub biz_name: String,
    pub biz_category: Option<String>,
    pub biz_logo: Option<String>,
    pub mode_dialog: bool,
    pub confirm: Option<Confirm>,
    pub about: bool,
}

impl ProfileState {
    pub fn open_business(&mut self, p: &Profile, creating: bool) {
        self.business = true;
        self.creating = creating;
        self.biz_name = p.business_name.clone().unwrap_or_default();
        self.biz_category = if creating { None } else { p.business_category.clone() };
        self.biz_logo = p.business_logo_url.clone();
    }

    fn open_edit(&mut self, p: &Profile) {
        self.edit = true;
        self.name = p.display_name.clone().unwrap_or_default();
        self.currency = p.currency();
        self.photo = p.photo_url.clone();
    }
}

pub fn show(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    header(app, ctx, ui);
    ui.add_space(16.0);
    if app.profile.deletion_scheduled_at.is_some() {
        let when = deletion_date(&app.profile);
        if crate::app::banner(ui, ic::WARNING, &format!("Your account is scheduled for deletion on {when}. Click to keep your account."), RED, RED_SOFT) {
            app.profile_ui.confirm = Some(Confirm::CancelDeletion);
        }
    }
    if ui.available_width() > 900.0 {
        ui.columns(2, |c| {
            settings_sections(app, ctx, &mut c[0]);
            support_section(app, ctx, &mut c[1]);
            c[1].add_space(14.0);
            sign_out(app, &mut c[1]);
        });
    } else {
        settings_sections(app, ctx, ui);
        ui.add_space(14.0);
        support_section(app, ctx, ui);
        ui.add_space(14.0);
        sign_out(app, ui);
    }
    ui.add_space(14.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new("Made with ❤ by the QuickBiz team").color(FAINT).size(12.5));
    });
}

fn deletion_date(p: &Profile) -> String {
    p.deletion_scheduled_at
        .as_deref()
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&chrono::Local).format("%d %B %Y").to_string())
        .unwrap_or_else(|| "a date soon".into())
}

fn header(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(WHITE)
        .stroke(Stroke::new(1.0_f32, LINE))
        .corner_radius(CornerRadius::same(22))
        .inner_margin(Margin::same(24))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let url = app.profile.photo_url.clone();
                let initial = app.profile.initial();
                let r = app.avatar(ui, url, &initial, 96.0, BRAND, Sense::click());
                // camera badge
                let badge = egui::Rect::from_center_size(r.rect.right_bottom() - egui::vec2(14.0, 14.0), egui::vec2(28.0, 28.0));
                ui.painter().circle_filled(badge.center(), 15.0, WHITE);
                ui.painter().circle_filled(badge.center(), 13.0, BRAND);
                ui::paint_icon(ui, badge, ic::CAMERA, 12.0, WHITE);
                let tip = if app.uploading == Some("profile") { "Uploading…" } else { "Change photo" };
                if r.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tip).clicked() {
                    app.pick_picture(ctx, "profile");
                }
                ui.add_space(14.0);
                ui.vertical(|ui| {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(app.profile.name()).font(bold_font(24.0)));
                        if app.profile.is_verified.unwrap_or(false) {
                            ui::icon(ui, ic::CHECK_CIRCLE, 18.0, BRAND);
                        }
                    });
                    let email = app.session.as_ref().map(|s| s.email.clone()).or_else(|| app.profile.email.clone()).unwrap_or_default();
                    ui.label(RichText::new(email).color(MUTED));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if app.profile.is_pro.unwrap_or(false) {
                            ui::pill(ui, "★ PRO", GOLD, GOLD_SOFT);
                        }
                        if app.profile.is_verified.unwrap_or(false) {
                            ui::pill(ui, "VERIFIED", BRAND, BRAND_SOFT);
                        }
                        let admin = app.profile.system_role.as_deref() == Some("admin");
                        let role = app.profile.role.clone().unwrap_or_else(|| "user".into()).to_uppercase();
                        if admin {
                            ui::pill(ui, &role, RED, RED_SOFT);
                        } else {
                            ui::pill(ui, &role, GREEN, GREEN_SOFT);
                        }
                        ui::pill(ui, &app.profile.tier.clone().unwrap_or_else(|| "new".into()).to_uppercase(), BRAND, BRAND_SOFT);
                    });
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(ui::secondary_button("Edit profile").min_size(egui::vec2(130.0, 40.0))).clicked() {
                        app.profile_ui.open_edit(&app.profile);
                    }
                });
            });
        });
}

fn list_card(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui::section_title(ui, title);
    egui::Frame::new()
        .fill(WHITE)
        .stroke(Stroke::new(1.0_f32, LINE))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(Margin::same(6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            add(ui);
        });
    ui.add_space(14.0);
}

fn settings_sections(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let business = app.business();
    let owner = app.profile.is_owner();
    let employee = app.profile.is_employee();
    let biz_name = app.profile.business_name.clone().unwrap_or_else(|| "Business".into());
    list_card(ui, if business { "Business settings" } else { "App settings" }, |ui| {
        if owner && business {
            let sub = app.profile.business_name.clone().unwrap_or_else(|| "Set your business name".into());
            if ui::menu_row(ui, ic::SHOP, BRAND, "Business profile", Some(&sub), None).clicked() {
                app.profile_ui.open_business(&app.profile, false);
            }
            if ui::menu_row(ui, ic::PEOPLE, BRAND, "Team management", Some("Add employees to your business"), None).clicked() {
                app.go(Tab::Team, ctx);
            }
            if ui::menu_row(ui, ic::DELETE, RED, "Delete business", Some("Remove business profile and data"), None).clicked() {
                app.profile_ui.confirm = Some(Confirm::DeleteBusiness);
            }
        }
        if employee && ui::menu_row(ui, ic::BACK, RED, "Leave business", Some(&format!("Current: {biz_name}")), None).clicked() {
            app.profile_ui.confirm = Some(Confirm::LeaveBusiness);
        }
        let sub = if employee {
            format!("Employee at {biz_name}")
        } else if owner && business {
            format!("Managing {biz_name}")
        } else if business {
            "Currently in Business mode".into()
        } else {
            "Currently in Personal mode".into()
        };
        let (glyph, color) = if business { (ic::SHOP, BRAND) } else { (ic::PERSON, GREEN) };
        if ui::menu_row(ui, glyph, color, "App mode", Some(&sub), None).clicked() {
            app.profile_ui.mode_dialog = true;
        }
    });

    let deleting = app.profile.deletion_scheduled_at.is_some();
    let currency = app.profile.currency();
    list_card(ui, "Account settings", |ui| {
        if ui::menu_row(ui, ic::PERSON, INK, "Edit profile", Some(&format!("Name, photo and currency ({currency})")), None).clicked() {
            app.profile_ui.open_edit(&app.profile);
        }
        if ui::menu_row(ui, ic::BELL, INK, "Notifications", Some("Manage your alerts and updates"), None).clicked() {
            app.go(Tab::Notifications, ctx);
        }
        if ui::menu_row(ui, ic::LOCK, INK, "Security", Some("Change password and 2FA"), None).clicked() {
            app.say("Security settings will be available in a future update.", true);
        }
        let (title, sub) = if deleting {
            ("Cancel deletion".to_string(), format!("Scheduled for {}", deletion_date(&app.profile)))
        } else {
            ("Delete account".to_string(), "Permanently remove your account and data".to_string())
        };
        if ui::menu_row(ui, ic::DELETE, RED, &title, Some(&sub), None).clicked() {
            app.profile_ui.confirm = Some(if deleting { Confirm::CancelDeletion } else { Confirm::DeleteAccount });
        }
    });
}

fn support_section(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let update = app.settings.windows_version.clone().filter(|v| is_newer_version(v, config::APP_VERSION));
    list_card(ui, "Support & legal", |ui| {
        let sub = match &update {
            Some(v) => format!("Version {v} is available"),
            None => format!("Current version: {}", config::APP_VERSION),
        };
        if ui::menu_row(ui, ic::REFRESH, BLUE, "Check for updates", Some(&sub), update.as_ref().map(|_| "New")).clicked() {
            if update.is_some() {
                open_url(if config::STORE_BUILD { config::STORE_URL } else { config::WEB_URL });
            } else {
                app.say("You have the latest version of QuickBiz.", true);
            }
        }
        if ui::menu_row(ui, ic::STAR, BRAND, "What's new", Some(&format!("See what changed in version {}", config::APP_VERSION)), None).clicked() {
            app.whats_new = true;
        }
        if ui::menu_row(ui, ic::GLOBE, INK, "Visit website", Some("quickbiz.vercel.app"), None).clicked() {
            open_url(config::WEB_URL);
        }
        if ui::menu_row(ui, ic::BULB, WARN, "Send feedback", Some("Report a problem or ask for a feature"), None).clicked() {
            app.support.start("feedback");
            app.go(Tab::Support, ctx);
        }
        if ui::menu_row(ui, ic::HELP, INK, "Help center", Some("Ask us anything and see our replies"), None).clicked() {
            app.support.start("question");
            app.go(Tab::Support, ctx);
        }
        if ui::menu_row(ui, ic::SHIELD, INK, "Privacy policy", None, None).clicked() {
            open_url(config::PRIVACY_URL);
        }
        if ui::menu_row(ui, ic::INFO, INK, "About QuickBiz", Some(&format!("Version {}", config::APP_VERSION)), None).clicked() {
            app.profile_ui.about = true;
        }
    });
}

fn sign_out(app: &mut App, ui: &mut egui::Ui) {
    let b = ui::soft_danger_button("Sign out").min_size(egui::vec2(ui.available_width(), 46.0));
    if ui.add(b).clicked() {
        app.profile_ui.confirm = Some(Confirm::SignOut);
    }
}

/// A picture finished uploading.
pub fn uploaded(app: &mut App, ctx: &egui::Context, kind: &str, url: String) {
    let uid = app.profile.uid.clone();
    if kind == "profile" {
        if app.profile_ui.edit {
            app.profile_ui.photo = Some(url);
        } else {
            app.profile.photo_url = Some(url.clone());
            app.write(ctx, "Profile photo updated.", move |c| c.update_me(&uid, json!({ "photoURL": url })));
        }
    } else if app.profile_ui.business {
        app.profile_ui.biz_logo = Some(url);
    } else {
        app.write(ctx, "Business logo updated.", move |c| c.update_me(&uid, json!({ "businessLogoURL": url })));
    }
}

/// All Profile dialogs. Called every frame while signed in, so the business dialog can also
/// open from the mode switch in the header.
pub fn dialogs(app: &mut App, ctx: &egui::Context) {
    edit_dialog(app, ctx);
    business_dialog(app, ctx);
    mode_dialog(app, ctx);
    about_dialog(app, ctx);
    confirm_dialog(app, ctx);
}

fn photo_picker(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui, url: Option<String>, initial: &str, kind: &'static str, label: &str) {
    ui.vertical_centered(|ui| {
        let r = app.avatar(ui, url, initial, 88.0, BRAND, Sense::click());
        let badge = egui::Rect::from_center_size(r.rect.right_bottom() - egui::vec2(12.0, 12.0), egui::vec2(26.0, 26.0));
        ui.painter().circle_filled(badge.center(), 14.0, WHITE);
        ui.painter().circle_filled(badge.center(), 12.0, BRAND);
        ui::paint_icon(ui, badge, ic::CAMERA, 11.0, WHITE);
        if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            app.pick_picture(ctx, kind);
        }
        let text = if app.uploading == Some(kind) { "Uploading…" } else { label };
        ui.label(RichText::new(text).color(MUTED).size(12.5));
    });
}

fn edit_dialog(app: &mut App, ctx: &egui::Context) {
    if !app.profile_ui.edit {
        return;
    }
    let mut save = false;
    let photo = app.profile_ui.photo.clone();
    let initial = app.profile_ui.name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| app.profile.initial());
    let (_, closed) = ui::dialog(ctx, "edit-profile", "Edit profile", 460.0, |ui| {
        ui.label(RichText::new("Your name is shown in your profile and on the entries you record.").color(MUTED));
        ui.add_space(12.0);
        photo_picker(app, ctx, ui, photo, &initial, "profile", "Change photo");
        ui.add_space(8.0);
        ui::field(ui, "Display name", &mut app.profile_ui.name, "Your name");
        ui.add_space(8.0);
        ui.label(RichText::new("DEFAULT CURRENCY").font(bold_font(11.5)).color(MUTED).extra_letter_spacing(1.0));
        ui.horizontal_wrapped(|ui| {
            for (code, sym) in CURRENCIES {
                if ui::chip(ui, &format!("{sym} {code}"), app.profile_ui.currency == code).clicked() {
                    app.profile_ui.currency = code.to_string();
                }
            }
        });
        ui.add_space(16.0);
        let ok = !app.profile_ui.name.trim().is_empty() && app.uploading.is_none();
        if ui.add_enabled(ok, ui::primary_button("Save profile").min_size(egui::vec2(ui.available_width(), 42.0))).clicked() {
            save = true;
        }
    });
    if closed {
        app.profile_ui.edit = false;
        return;
    }
    if save {
        let s = &app.profile_ui;
        let body = json!({ "displayName": s.name.trim(), "photoURL": s.photo, "currencyCode": s.currency });
        app.profile.display_name = Some(s.name.trim().to_string());
        app.profile.photo_url = s.photo.clone();
        app.profile.currency_code = Some(s.currency.clone());
        app.profile_ui.edit = false;
        let uid = app.profile.uid.clone();
        app.write(ctx, "Profile updated.", move |c| c.update_me(&uid, body));
    }
}

fn business_dialog(app: &mut App, ctx: &egui::Context) {
    if !app.profile_ui.business {
        return;
    }
    let creating = app.profile_ui.creating;
    let mut save = false;
    let logo = app.profile_ui.biz_logo.clone();
    let title = if creating { "Create your business" } else { "Business profile" };
    let (_, closed) = ui::dialog(ctx, "business-profile", title, 480.0, |ui| {
        let intro = if creating {
            "Business mode keeps your shop's sales, stock and team separate from your personal money. Give it a name to start."
        } else {
            "Your business name is visible to your employees and on your records."
        };
        ui.label(RichText::new(intro).color(MUTED));
        ui.add_space(12.0);
        photo_picker(app, ctx, ui, logo, "B", "business", "Business logo");
        ui.add_space(8.0);
        ui::field(ui, "Business name *", &mut app.profile_ui.biz_name, "e.g. Acme Mobile Shop");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Business category *").font(bold_font(13.0)).color(MUTED));
            if app.profile_ui.biz_category.is_none() {
                ui.label(RichText::new("(please choose one)").color(RED).size(12.5));
            }
        });
        ui.horizontal_wrapped(|ui| {
            for cat in BUSINESS_CATEGORIES {
                let on = app.profile_ui.biz_category.as_deref() == Some(cat);
                if ui::chip(ui, cat, on).clicked() {
                    app.profile_ui.biz_category = Some(cat.to_string());
                }
            }
        });
        ui.add_space(16.0);
        let ok = !app.profile_ui.biz_name.trim().is_empty() && app.profile_ui.biz_category.is_some() && app.uploading.is_none();
        let label = if creating { "Create business" } else { "Save changes" };
        if ui.add_enabled(ok, ui::primary_button(label).min_size(egui::vec2(ui.available_width(), 42.0))).clicked() {
            save = true;
        }
    });
    if closed {
        app.profile_ui.business = false;
        return;
    }
    if !save {
        return;
    }
    let s = &app.profile_ui;
    let uid = app.profile.uid.clone();
    let name = s.biz_name.trim().to_string();
    let category = s.biz_category.clone().unwrap_or_default();
    let logo = s.biz_logo.clone();
    app.profile_ui.business = false;
    if creating && !app.profile.has_business() {
        let body = json!({
            "role": "owner",
            "businessId": uid,
            "businessOwnerId": uid,
            "businessName": name,
            "businessCategory": category,
            "businessLogoURL": logo,
        });
        // show business mode straight away; a failed save puts it back when the profile reloads
        app.profile.role = Some("owner".into());
        app.profile.business_id = Some(uid.clone());
        app.profile.business_owner_id = Some(uid.clone());
        app.profile.business_name = Some(name);
        app.mode = "business".into();
        app.txs.clear();
        app.products.clear();
        app.write(ctx, "Business created. You're now in Business mode.", move |c| c.update_me(&uid, body));
    } else {
        app.profile.business_name = Some(name.clone());
        let body = json!({ "businessName": name, "businessCategory": category, "businessLogoURL": logo });
        app.write(ctx, "Business profile updated.", move |c| c.update_me(&uid, body));
    }
}

fn mode_dialog(app: &mut App, ctx: &egui::Context) {
    if !app.profile_ui.mode_dialog {
        return;
    }
    let mut pick: Option<&str> = None;
    let business = app.business();
    let employee = app.profile.is_employee();
    let biz = app.profile.business_name.clone();
    let (_, closed) = ui::dialog(ctx, "mode", "Switch app mode", 460.0, |ui| {
        ui.label(RichText::new("Choose how you want to use QuickBiz right now.").color(MUTED));
        ui.add_space(12.0);
        if mode_option(ui, ic::PERSON, GREEN, "Personal mode", "Track your personal income and expenses.", !business) {
            pick = Some("personal");
        }
        ui.add_space(8.0);
        let sub = if employee {
            format!("Open the {} dashboard.", biz.clone().unwrap_or_else(|| "company".into()))
        } else if let Some(b) = &biz {
            format!("Manage {b}.")
        } else {
            "Create or manage your business.".into()
        };
        if mode_option(ui, ic::SHOP, BRAND, "Business mode", &sub, business) {
            pick = Some("business");
        }
        if biz.is_none() && !employee {
            ui.add_space(10.0);
            crate::app::banner(ui, ic::INFO, "Switching to Business mode for the first time asks you to create your business profile.", BLUE, BLUE_SOFT);
        }
    });
    if let Some(m) = pick {
        app.profile_ui.mode_dialog = false;
        app.set_mode(ctx, m);
    } else if closed {
        app.profile_ui.mode_dialog = false;
    }
}

fn mode_option(ui: &mut egui::Ui, glyph: &str, color: egui::Color32, title: &str, sub: &str, on: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 72.0), Sense::click());
    let fill = if on { tint(color, 0.06) } else if resp.hovered() { SURFACE } else { WHITE };
    ui.painter().rect(rect, CornerRadius::same(14), fill, Stroke::new(if on { 2.0_f32 } else { 1.0_f32 }, if on { color } else { LINE }), egui::StrokeKind::Inside);
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(egui::vec2(14.0, 0.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
    ui::icon_circle(&mut child, glyph, color, 46.0);
    child.vertical(|ui| {
        ui.add_space(16.0);
        ui.spacing_mut().item_spacing.y = 1.0;
        ui.label(RichText::new(title).font(bold_font(15.0)));
        ui.label(RichText::new(sub).color(MUTED).size(12.5));
    });
    if on {
        let c = egui::Rect::from_center_size(egui::pos2(rect.right() - 26.0, rect.center().y), egui::vec2(22.0, 22.0));
        ui::paint_icon(ui, c, ic::CHECK_CIRCLE, 18.0, color);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

fn about_dialog(app: &mut App, ctx: &egui::Context) {
    if !app.profile_ui.about {
        return;
    }
    let (_, closed) = ui::dialog(ctx, "about", "About QuickBiz", 400.0, |ui| {
        ui.vertical_centered(|ui| {
            if let Some(tex) = crate::logo::texture(ctx) {
                ui.image((tex.id(), egui::vec2(64.0, 64.0)));
            }
            ui.add_space(6.0);
            ui.label(RichText::new("QuickBiz for Windows").font(bold_font(17.0)));
            ui.label(RichText::new(format!("Version {}", config::APP_VERSION)).color(MUTED));
            ui.add_space(8.0);
            ui.label(RichText::new("Your smart business finance companion.").color(MUTED));
            ui.add_space(4.0);
            ui.label(RichText::new("Your data syncs with the QuickBiz phone app.").color(MUTED));
        });
    });
    if closed {
        app.profile_ui.about = false;
    }
}

fn confirm_dialog(app: &mut App, ctx: &egui::Context) {
    let Some(which) = app.profile_ui.confirm else { return };
    let biz = app.profile.business_name.clone().unwrap_or_else(|| "this business".into());
    let (title, text, yes, danger) = match which {
        Confirm::SignOut => ("Sign out?", "You will need your email and password to sign in again.".to_string(), "Sign out", true),
        Confirm::LeaveBusiness => ("Leave business?", format!("Are you sure you want to leave {biz}? You will lose access to all business data."), "Leave", true),
        Confirm::DeleteBusiness => (
            "Delete business?",
            format!("This permanently deletes \"{biz}\": all business transactions, products and stock history, and removes your employees from it. This cannot be undone."),
            "Delete everything",
            true,
        ),
        Confirm::DeleteAccount => (
            "Request account deletion?",
            "Your account and all its data will be permanently deleted in 30 days. You can cancel before then by signing in again. You will be signed out now.".to_string(),
            "Schedule deletion",
            true,
        ),
        Confirm::CancelDeletion => (
            "Keep your account?",
            "Your account is scheduled for deletion. Do you want to cancel the request and keep your account?".to_string(),
            "Yes, keep my account",
            false,
        ),
    };
    let Some(answer) = ui::confirm(ctx, "profile-confirm", title, &text, yes, danger) else { return };
    app.profile_ui.confirm = None;
    if !answer {
        return;
    }
    let uid = app.profile.uid.clone();
    match which {
        Confirm::SignOut => app.sign_out(),
        Confirm::LeaveBusiness => app.write(ctx, "You have left the business.", |c| c.leave_business()),
        Confirm::DeleteBusiness => app.write(ctx, "Your business and its data have been removed.", |c| c.delete_business()),
        Confirm::DeleteAccount => {
            let when = (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339();
            app.spawn(ctx, move |c, tx| {
                let _ = tx.send(crate::app::Msg::DeletionScheduled(c.update_me(&uid, json!({ "deletionScheduledAt": when }))));
            });
        }
        Confirm::CancelDeletion => {
            app.profile.deletion_scheduled_at = None;
            app.write(ctx, "Your account deletion request has been cancelled.", move |c| c.update_me(&uid, json!({ "deletionScheduledAt": null })));
        }
    }
}
