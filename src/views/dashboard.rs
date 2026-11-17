//! Home screen: balance card, quick actions, this month's spending, stock insights and recent activity.
//! Mirrors the mobile dashboard, laid out in two columns for a wide window.

use crate::app::{App, Tab};
use crate::theme::*;
use crate::ui;
use chrono::{Datelike, Local};
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Sense, Stroke};

const MASK: &str = "••••••";

pub fn show(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let perms = app.perms();
    let employee = app.profile.is_employee();
    let business = app.business();
    let restricted = business && !perms.can_view_reports;
    let code = app.profile.currency();

    // Employees see their own entries; totals use only the account currency, as on the phone.
    let me = app.profile.uid.clone();
    let mine: Vec<_> = app.txs.iter().filter(|t| !employee || !business || t.created_by.as_deref() == Some(me.as_str())).cloned().collect();
    let summary: Vec<_> = mine.iter().filter(|t| t.currency_code.as_deref().map(|c| c == code).unwrap_or(true)).collect();

    let today = Local::now().date_naive();
    let (mut income, mut expense, mut m_income, mut m_expense, mut t_expense) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for t in &summary {
        let (inc, amt) = (t.is_income(), t.amount);
        if inc { income += amt } else { expense += amt }
        if let Some(d) = t.day() {
            if d.year() == today.year() && d.month() == today.month() {
                if inc { m_income += amt } else { m_expense += amt }
            }
            if d == today && !inc {
                t_expense += amt;
            }
        }
    }
    let mask = restricted || app.prefs.hide_balance;
    let money = |v: f64| if mask { MASK.to_string() } else { app.money(v) };

    hero(app, ui, &money(income - expense), &money(income), &money(expense), restricted, business && employee);
    ui.add_space(16.0);
    quick_actions(app, ctx, ui);
    ui.add_space(16.0);

    let left_right = ui.available_width() > 860.0;
    let pulse_money = |v: f64| if restricted { MASK.to_string() } else { app.money(v) };
    let pulse_vals = (pulse_money(t_expense), pulse_money(m_expense), pulse_money(m_income));
    if left_right {
        ui.columns(2, |c| {
            recent(app, ctx, &mut c[0], &mine);
            pulse(&mut c[1], m_income, m_expense, &pulse_vals, restricted);
            c[1].add_space(14.0);
            owed(app, ctx, &mut c[1], &mine, restricted);
            inventory(app, ctx, &mut c[1]);
        });
    } else {
        pulse(ui, m_income, m_expense, &pulse_vals, restricted);
        ui.add_space(14.0);
        owed(app, ctx, ui, &mine, restricted);
        inventory(app, ctx, ui);
        ui.add_space(14.0);
        recent(app, ctx, ui, &mine);
    }
}

/// The blue balance card with decorative circles, an eye to hide amounts, and income/expense chips.
fn hero(app: &mut App, ui: &mut egui::Ui, balance: &str, income: &str, expense: &str, restricted: bool, staff: bool) {
    let resp = egui::Frame::new()
        .fill(BRAND)
        .corner_radius(CornerRadius::same(24))
        .inner_margin(Margin::same(24))
        .shadow(egui::epaint::Shadow { offset: [0, 10], blur: 28, spread: 0, color: Color32::from_rgba_unmultiplied(45, 98, 237, 70) })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let rect = ui.max_rect().expand(24.0);
            let p = ui.painter().with_clip_rect(rect);
            p.circle_filled(egui::pos2(rect.right() - 40.0, rect.top() + 20.0), 120.0, tint_on(BRAND, 0.10));
            p.circle_filled(egui::pos2(rect.left() + 30.0, rect.bottom() - 10.0), 64.0, tint_on(BRAND, 0.07));
            ui.horizontal(|ui| {
                let label = if staff { "BUSINESS BALANCE" } else { "TOTAL BALANCE" };
                ui.label(RichText::new(label).font(bold_font(12.0)).color(tint_on(BRAND, 0.8)).extra_letter_spacing(1.2));
                if !restricted {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let glyph = if app.prefs.hide_balance { ic::EYE_OFF } else { ic::EYE };
                        let b = egui::Button::new(ui::icon_text(glyph, 14.0, WHITE)).fill(tint_on(BRAND, 0.2)).stroke(Stroke::NONE).corner_radius(CornerRadius::same(255)).min_size(egui::vec2(34.0, 34.0));
                        let tip = if app.prefs.hide_balance { "Show amounts" } else { "Hide amounts" };
                        if ui.add(b).on_hover_text(tip).clicked() {
                            app.prefs.hide_balance = !app.prefs.hide_balance;
                            app.save_prefs();
                        }
                    });
                }
            });
            ui.label(RichText::new(balance).font(bold_font(40.0)).color(WHITE));
            ui.add_space(10.0);
            egui::Frame::new().fill(tint_on(BRAND, 0.14)).corner_radius(CornerRadius::same(16)).inner_margin(Margin::same(12)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.columns(2, |c| {
                    hero_stat(&mut c[0], ic::DOWN, Color32::from_rgb(0x4A, 0xDE, 0x80), "Income", income);
                    hero_stat(&mut c[1], ic::UP, Color32::from_rgb(0xFC, 0xA5, 0xA5), "Expense", expense);
                });
            });
            if restricted {
                ui.add_space(6.0);
                ui.label(RichText::new("Ask the business owner for report access to see these amounts.").color(tint_on(BRAND, 0.78)).size(12.5));
            }
        });
    let _ = resp;
}

fn hero_stat(ui: &mut egui::Ui, glyph: &str, color: Color32, label: &str, value: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 17.0, tint_on(BRAND, 0.22));
        ui::paint_icon(ui, rect, glyph, 14.0, color);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(RichText::new(label).color(tint_on(BRAND, 0.78)).size(12.5));
            ui.label(RichText::new(value).font(bold_font(17.0)).color(WHITE));
        });
    });
}

fn quick_actions(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let perms = app.perms();
    let business = app.business();
    let mut actions: Vec<(&str, &str, Color32, u8)> = vec![];
    if perms.can_manage_transactions {
        actions.push((ic::ADD, if business { "New sale" } else { "Add income" }, GREEN, 0));
        actions.push((ic::REMOVE, "Add expense", RED, 1));
    }
    if business && perms.can_view_stock {
        actions.push((ic::BOX, "Stock", BRAND, 2));
    }
    if !business || app.profile.is_owner() {
        actions.push((ic::CHART, "Reports", BLUE, 3));
    }
    if actions.is_empty() {
        return;
    }
    let n = actions.len();
    ui.columns(n, |c| {
        for (i, (glyph, label, color, id)) in actions.iter().enumerate() {
            let ui = &mut c[i];
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 92.0), Sense::click());
            let fill = if resp.hovered() { tint(*color, 0.05) } else { WHITE };
            ui.painter().rect(rect, CornerRadius::same(18), fill, Stroke::new(1.0_f32, if resp.hovered() { tint(*color, 0.35) } else { LINE }), egui::StrokeKind::Inside);
            let icon_rect = egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.top() + 34.0), egui::vec2(44.0, 44.0));
            ui.painter().rect_filled(icon_rect, CornerRadius::same(14), tint(*color, 0.12));
            ui::paint_icon(ui, icon_rect, glyph, 19.0, *color);
            ui.painter().text(egui::pos2(rect.center().x, rect.bottom() - 18.0), egui::Align2::CENTER_CENTER, *label, bold_font(13.5), INK);
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                match id {
                    0 | 1 => {
                        app.finance.income = *id == 0;
                        app.finance.repair = false;
                        app.finance.inited = true;
                        app.finance.focus = true;
                        app.go(Tab::Finance, ctx);
                    }
                    2 => app.go(Tab::Inventory, ctx),
                    _ => app.go(Tab::Reports, ctx),
                }
            }
        }
    });
}

fn pulse(ui: &mut egui::Ui, m_income: f64, m_expense: f64, vals: &(String, String, String), restricted: bool) {
    ui::card(ui, |ui| {
        let spent = if m_income > 0.0 { Some(((m_expense / m_income) * 100.0).round().min(100.0) as u32) } else { None };
        let color = match spent {
            None => FAINT,
            Some(p) if p < 60 => GREEN,
            Some(p) if p < 85 => WARN,
            _ => RED,
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new("This month").font(bold_font(16.5)));
            if let (Some(p), false) = (spent, restricted) {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui::pill(ui, &format!("{p}% spent"), color, tint(color, 0.12));
                });
            }
        });
        ui.add_space(8.0);
        let (track, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 10.0), Sense::hover());
        ui.painter().rect_filled(track, CornerRadius::same(5), SURFACE_MUTED);
        let share = if restricted { 0.0 } else { spent.unwrap_or(0) as f32 / 100.0 };
        if share > 0.0 {
            ui.painter().rect_filled(track.with_max_x(track.left() + track.width() * share), CornerRadius::same(5), color);
        }
        ui.add_space(6.0);
        let msg = if restricted {
            "Ask the business owner for report access to see this summary.".to_string()
        } else if let Some(p) = spent {
            format!("You've used {p}% of this month's income.")
        } else if m_expense > 0.0 {
            "No income recorded this month yet.".into()
        } else {
            "Nothing recorded this month yet.".into()
        };
        ui.label(RichText::new(msg).color(MUTED).size(13.0));
        ui.add_space(10.0);
        ui::divider(ui);
        ui.add_space(10.0);
        ui.columns(3, |c| {
            for (i, (label, value, col)) in [("Today's expense", &vals.0, RED), ("Spent", &vals.1, INK), ("Earned", &vals.2, GREEN)].into_iter().enumerate() {
                c[i].label(RichText::new(label).color(MUTED).size(12.5));
                c[i].label(RichText::new(value).font(bold_font(16.0)).color(col));
            }
        });
    });
}

fn inventory(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    if !app.business() || app.products.is_empty() {
        return;
    }
    let perms = app.perms();
    let value: f64 = app.products.iter().map(|p| p.stock_quantity * p.cost).sum();
    let revenue: f64 = app.products.iter().map(|p| p.stock_quantity * p.price).sum();
    let low = app.products.iter().filter(|p| p.is_low()).count();
    let (stock_value, potential) = if perms.can_view_price { (app.money(value), app.money(revenue)) } else { (MASK.into(), MASK.into()) };
    let (_, manage) = ui::titled_card(ui, "Inventory insights", Some("Manage"), |ui| {
        ui.columns(2, |c| {
            for (i, (label, v, col)) in [("Stock value", &stock_value, BRAND), ("Potential revenue", &potential, GREEN)].into_iter().enumerate() {
                egui::Frame::new().fill(SURFACE).corner_radius(CornerRadius::same(14)).inner_margin(Margin::same(14)).show(&mut c[i], |ui| {
                    ui.set_width(ui.available_width());
                    let (dot, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), Sense::hover());
                    ui.painter().circle_filled(dot.center(), 4.0, col);
                    ui.label(RichText::new(label).color(MUTED).size(12.5));
                    ui.label(RichText::new(v).font(bold_font(18.0)));
                });
            }
        });
        if perms.can_view_stock && low > 0 {
            ui.add_space(10.0);
            let r = egui::Frame::new().fill(RED_SOFT).stroke(Stroke::new(1.0_f32, tint(RED, 0.25))).corner_radius(CornerRadius::same(14)).inner_margin(Margin::symmetric(12, 10)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui::icon(ui, ic::WARNING, 15.0, RED);
                    let s = if low == 1 { "1 item is low on stock".to_string() } else { format!("{low} items are low on stock") };
                    ui.label(RichText::new(s).font(bold_font(14.0)).color(RED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui::icon(ui, ic::CHEVRON_RIGHT, 11.0, RED);
                    });
                });
            });
            if r.response.interact(Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                app.inventory.low_only = true;
                app.go(Tab::Inventory, ctx);
            }
        }
    });
    if manage {
        app.go(Tab::Inventory, ctx);
    }
}

fn recent(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui, txs: &[crate::model::Transaction]) {
    let (_, see_all) = ui::titled_card(ui, "Recent activity", Some("See all"), |ui| {
        if txs.is_empty() {
            ui::empty_state(ui, ic::CARD, "No transactions yet", "Add your first entry and your balance will appear here.");
            if app.perms().can_manage_transactions {
                ui.vertical_centered(|ui| {
                    if ui.add(ui::primary_button("Add your first entry").min_size(egui::vec2(200.0, 40.0))).clicked() {
                        app.finance.income = app.business();
                        app.finance.inited = true;
                        app.finance.focus = true;
                        app.go(Tab::Finance, ctx);
                    }
                });
            }
            return;
        }
        for (i, t) in txs.iter().take(7).enumerate() {
            if i > 0 {
                ui::divider(ui);
            }
            super::finance::row(app, ui, t, false);
        }
    });
    if see_all {
        app.go(Tab::Finance, ctx);
    }
}

/// Money customers still owe (repairs, credit sales, jobs), with a button to record a payment.
fn owed(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui, txs: &[crate::model::Transaction], restricted: bool) {
    if !app.business() {
        return;
    }
    let code = app.profile.currency();
    let list: Vec<_> = txs.iter().filter(|t| t.due_amount.unwrap_or(0.0) > 0.0).cloned().collect();
    if list.is_empty() {
        return;
    }
    let total: f64 = list.iter().filter(|t| t.currency_code.as_deref().map(|c| c == code).unwrap_or(true)).filter_map(|t| t.due_amount).sum();
    let can_pay = app.perms().can_manage_transactions;
    let (_, see_all) = ui::titled_card(ui, "Customers owe you", Some("See all"), |ui| {
        ui.label(RichText::new("Still to collect").color(MUTED).size(12.5));
        ui.label(RichText::new(if restricted { MASK.to_string() } else { app.money(total) }).font(bold_font(22.0)).color(RED));
        ui.add_space(4.0);
        for t in list.iter().take(4) {
            ui::divider(ui);
            ui.horizontal(|ui| {
                ui.set_min_height(40.0);
                ui::icon_circle(ui, ic::PERSON, AMBER, 30.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.add_space(3.0);
                    ui.label(RichText::new(t.customer_name.clone().filter(|n| !n.is_empty()).unwrap_or_else(|| t.category.clone().unwrap_or_else(|| "Customer".into()))).font(bold_font(13.5)));
                    ui.label(RichText::new(t.kind().label()).color(MUTED).size(12.0));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if can_pay && ui.add(ui::secondary_button("Record payment").min_size(egui::vec2(0.0, 30.0))).clicked() {
                        app.finance.pay_input.clear();
                        app.finance.pay_for = Some(t.clone());
                    }
                    let v = if restricted { MASK.to_string() } else { app.money(t.due_amount.unwrap_or(0.0)) };
                    ui.label(RichText::new(v).font(bold_font(14.0)).color(RED));
                });
            });
        }
    });
    ui.add_space(14.0);
    if see_all {
        app.go(Tab::Finance, ctx);
    }
    super::finance::payment_dialog(app, ctx);
}
