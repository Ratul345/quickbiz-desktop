//! Team: business owners invite employees by email and choose what each one may see and do.
//! Every change goes through the same database functions as the mobile app, which check you own the business.

use crate::api::Res;
use crate::app::{App, Msg, Tab};
use crate::model::*;
use crate::theme::*;
use crate::ui;
use eframe::egui::{self, RichText, Sense};

#[derive(Clone, Copy, PartialEq)]
pub enum Action {
    Promote,
    Remove,
}

#[derive(Default)]
pub struct TeamState {
    pub email: String,
    pub searching: bool,
    pub found: Option<FoundUser>,
    pub editing: Option<(String, String, Permissions)>,
    pub confirm: Option<(Action, String, String)>,
}

const PERMS: [(&str, &str); 5] = [
    ("View prices", "See product prices and stock value"),
    ("View stock", "See inventory levels"),
    ("Manage inventory", "Add products and change stock"),
    ("Manage transactions", "Record sales and expenses"),
    ("View reports", "See business totals and reports"),
];

fn perm_mut(p: &mut Permissions, i: usize) -> &mut bool {
    match i {
        0 => &mut p.can_view_price,
        1 => &mut p.can_view_stock,
        2 => &mut p.can_manage_inventory,
        3 => &mut p.can_manage_transactions,
        _ => &mut p.can_view_reports,
    }
}

pub fn show(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    if ui::back_link(ui, "Back to Profile") {
        app.go(Tab::Profile, ctx);
    }
    if !app.profile.is_owner() || !app.business() {
        ui::card(ui, |ui| {
            ui::empty_state(ui, ic::PEOPLE, "Only business owners manage a team", "Switch to Business mode as the owner to invite employees.");
        });
        return;
    }
    let wide = ui.available_width() > 900.0;
    if wide {
        ui.columns(2, |c| {
            invite_card(app, ctx, &mut c[0]);
            c[0].add_space(14.0);
            how_it_works(&mut c[0]);
            members(app, &mut c[1]);
        });
    } else {
        invite_card(app, ctx, ui);
        ui.add_space(14.0);
        members(app, ui);
    }
}

fn invite_card(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui::titled_card(ui, "Invite an employee", None, |ui| {
        ui.label(RichText::new("They need a QuickBiz account first. They join once they accept the invitation in their app.").color(MUTED));
        ui.add_space(8.0);
        let r = ui::field(ui, "Employee email", &mut app.team.email, "name@example.com");
        let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        ui.add_space(10.0);
        let label = if app.team.searching { "Searching…" } else { "Find and invite" };
        let go = ui.add_enabled(!app.team.searching, ui::primary_button(label).min_size(egui::vec2(ui.available_width(), 42.0))).clicked();
        if (go || enter) && !app.team.searching {
            search(app, ctx);
        }
    });
}

fn how_it_works(ui: &mut egui::Ui) {
    ui::titled_card(ui, "How it works", None, |ui| {
        for (glyph, text) in [
            (ic::MAIL, "Invite someone by the email they use for QuickBiz."),
            (ic::CHECK_CIRCLE, "They accept the invitation in their own app. Invitations last 7 days."),
            (ic::LOCK, "You decide what each employee can see and do."),
            (ic::BACK, "Employees can leave at any time, and you can remove them."),
        ] {
            ui.horizontal(|ui| {
                ui::icon_box(ui, glyph, BRAND, 30.0);
                ui.label(RichText::new(text).color(MUTED));
            });
            ui.add_space(4.0);
        }
    });
}

fn members(app: &mut App, ui: &mut egui::Ui) {
    let list = app.employees.clone();
    ui::titled_card(ui, &format!("Your team ({})", list.len()), None, |ui| {
        if list.is_empty() {
            ui::empty_state(ui, ic::PEOPLE, "No employees yet", "Invite your first employee with their email address.");
            return;
        }
        for (i, e) in list.iter().enumerate() {
            if i > 0 {
                ui::divider(ui);
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let initial = e.name().chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| "E".into());
                app.avatar(ui, e.photo_url.clone(), &initial, 42.0, BRAND, Sense::hover());
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(RichText::new(e.name()).font(bold_font(14.5)));
                    ui.label(RichText::new(e.email.clone().unwrap_or_default()).color(MUTED).size(12.5));
                });
            });
            let p = e.permissions.unwrap_or_default();
            ui.horizontal_wrapped(|ui| {
                ui.add_space(50.0);
                let mut any = false;
                for (i, (label, _)) in PERMS.iter().enumerate() {
                    let mut q = p;
                    if *perm_mut(&mut q, i) {
                        any = true;
                        ui::pill(ui, label, BRAND, BRAND_SOFT);
                    }
                }
                if !any {
                    ui::pill(ui, "No access yet", MUTED, SURFACE_MUTED);
                }
            });
            ui.horizontal(|ui| {
                ui.add_space(50.0);
                if ui.add(ui::secondary_button("Permissions").min_size(egui::vec2(0.0, 34.0))).clicked() {
                    app.team.editing = Some((e.uid.clone(), e.name(), p));
                }
                if ui.add(ui::secondary_button("Make owner").min_size(egui::vec2(0.0, 34.0))).clicked() {
                    app.team.confirm = Some((Action::Promote, e.uid.clone(), e.name()));
                }
                if ui.add(ui::soft_danger_button("Remove").min_size(egui::vec2(0.0, 34.0))).clicked() {
                    app.team.confirm = Some((Action::Remove, e.uid.clone(), e.name()));
                }
            });
            ui.add_space(6.0);
        }
    });
}

fn search(app: &mut App, ctx: &egui::Context) {
    let email = app.team.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        app.say("Enter the employee's email address.", false);
        return;
    }
    app.team.searching = true;
    app.spawn(ctx, move |c, tx| {
        let _ = tx.send(Msg::Found(c.find_user(&email)));
    });
}

/// The email lookup finished.
pub fn found(app: &mut App, _ctx: &egui::Context, r: Res<Option<FoundUser>>) {
    app.team.searching = false;
    match r {
        Err(e) => app.say(e, false),
        Ok(None) => app.say("No user found with this email. Make sure they have registered an account first.", false),
        Ok(Some(u)) if u.uid == app.profile.uid => app.say("You cannot invite yourself.", false),
        Ok(Some(u)) if u.role.as_deref() == Some("employee") => app.say("This user is already an employee of a business.", false),
        Ok(Some(u)) if u.role.as_deref() == Some("owner") => {
            app.say("This user already owns a business. You can only invite people who have not created a business.", false)
        }
        Ok(Some(u)) => app.team.found = Some(u),
    }
}

pub fn dialogs(app: &mut App, ctx: &egui::Context) {
    invite_dialog(app, ctx);
    permissions_dialog(app, ctx);
    action_dialog(app, ctx);
}

fn invite_dialog(app: &mut App, ctx: &egui::Context) {
    let Some(u) = app.team.found.clone() else { return };
    let who = u.display_name.clone().filter(|n| !n.is_empty()).or(u.email.clone()).unwrap_or_else(|| "this person".into());
    let biz = app.profile.business_name.clone().unwrap_or_else(|| "your business".into());
    let text = format!("Invite {who} to join {biz}? They will see the invitation the next time they open QuickBiz.");
    let Some(yes) = ui::confirm(ctx, "invite", "Send invitation?", &text, "Send invitation", false) else { return };
    app.team.found = None;
    if yes {
        app.team.email.clear();
        let (uid, name) = (u.uid.clone(), biz);
        app.write(ctx, "Invitation sent. They join your team once they accept.", move |c| c.invite(&uid, &name));
    }
}

fn permissions_dialog(app: &mut App, ctx: &egui::Context) {
    let Some((uid, name, mut perms)) = app.team.editing.clone() else { return };
    let mut save = false;
    let (_, closed) = ui::dialog(ctx, "permissions", &format!("Permissions for {name}"), 460.0, |ui| {
        ui.label(RichText::new("Choose what this employee can see and do in your business.").color(MUTED));
        ui.add_space(10.0);
        for (i, (title, sub)) in PERMS.iter().enumerate() {
            if i > 0 {
                ui::divider(ui);
            }
            ui.horizontal(|ui| {
                ui.set_min_height(52.0);
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(RichText::new(*title).font(bold_font(14.5)));
                    ui.label(RichText::new(*sub).color(MUTED).size(12.5));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui::toggle(ui, perm_mut(&mut perms, i));
                });
            });
        }
        ui.add_space(14.0);
        if ui.add(ui::primary_button("Save permissions").min_size(egui::vec2(ui.available_width(), 42.0))).clicked() {
            save = true;
        }
    });
    if closed {
        app.team.editing = None;
        return;
    }
    app.team.editing = Some((uid.clone(), name, perms));
    if save {
        app.team.editing = None;
        if let Some(e) = app.employees.iter_mut().find(|e| e.uid == uid) {
            e.permissions = Some(perms);
        }
        app.write(ctx, "Permissions updated.", move |c| c.set_permissions(&uid, &perms));
    }
}

fn action_dialog(app: &mut App, ctx: &egui::Context) {
    let Some((action, uid, name)) = app.team.confirm.clone() else { return };
    let (title, text, yes) = match action {
        Action::Promote => (
            "Make owner?",
            format!("{name} will become an owner and get full access to the business. This is meant for handing over the business."),
            "Make owner",
        ),
        Action::Remove => ("Remove employee?", format!("{name} will lose access to the business data."), "Remove"),
    };
    let Some(yes_clicked) = ui::confirm(ctx, "team-action", title, &text, yes, true) else { return };
    app.team.confirm = None;
    if !yes_clicked {
        return;
    }
    match action {
        Action::Promote => app.write(ctx, "They are now an owner.", move |c| c.promote(&uid)),
        Action::Remove => app.write(ctx, "Employee removed.", move |c| c.remove_employee(&uid)),
    }
}
