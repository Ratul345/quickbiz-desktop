use crate::app::{card, heading, stat, App};
use crate::theme::*;
use chrono::{Datelike, Local};
use eframe::egui::{self, RichText};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    heading(ui, "Dashboard");
    let today = Local::now().date_naive();
    let (mut income, mut expense, mut m_income, mut m_expense) = (0.0, 0.0, 0.0, 0.0);
    for t in &app.txs {
        let (inc, amt) = (t.is_income(), t.amount);
        if inc { income += amt } else { expense += amt }
        if let Some(d) = t.day() {
            if d.year() == today.year() && d.month() == today.month() {
                if inc { m_income += amt } else { m_expense += amt }
            }
        }
    }
    let low: Vec<_> = app.products.iter().filter(|p| p.is_low()).collect();
    let due: f64 = app.txs.iter().filter(|t| t.status.as_deref() != Some("cancelled")).filter_map(|t| t.due_amount).sum();

    ui.columns(3, |c| {
        stat(&mut c[0], "Balance", &app.money(income - expense), if income >= expense { GREEN } else { RED });
        stat(&mut c[1], "Total income", &app.money(income), GREEN);
        stat(&mut c[2], "Total expenses", &app.money(expense), RED);
    });
    ui.add_space(10.0);
    ui.columns(3, |c| {
        stat(&mut c[0], "Income this month", &app.money(m_income), GREEN);
        stat(&mut c[1], "Expenses this month", &app.money(m_expense), RED);
        if app.mode == "business" {
            stat(&mut c[2], "Customers still owe", &app.money(due), if due > 0.0 { AMBER } else { MUTED });
        } else {
            stat(&mut c[2], "Saved this month", &app.money(m_income - m_expense), BRAND);
        }
    });
    ui.add_space(14.0);

    ui.columns(2, |c| {
        card(&mut c[0], |ui| {
            ui.label(RichText::new("Recent activity").font(bold_font(16.0)));
            ui.add_space(4.0);
            if app.txs.is_empty() {
                ui.label(RichText::new("No transactions yet. Add your first one in Finance.").color(MUTED));
            }
            for t in app.txs.iter().take(8) {
                ui.horizontal(|ui| {
                    ui.label(t.category.clone().unwrap_or_else(|| "Uncategorised".into()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let sign = if t.is_income() { "+" } else { "-" };
                        let color = if t.is_income() { GREEN } else { RED };
                        ui.label(RichText::new(format!("{sign}{}", app.money(t.amount))).color(color));
                    });
                });
            }
        });
        card(&mut c[1], |ui| {
            ui.label(RichText::new("Low stock").font(bold_font(16.0)));
            ui.add_space(4.0);
            if low.is_empty() {
                ui.label(RichText::new("All products are well stocked.").color(MUTED));
            }
            for p in low.iter().take(8) {
                ui.horizontal(|ui| {
                    ui.label(&p.name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{} left", p.stock_quantity)).color(AMBER));
                    });
                });
            }
        });
    });
}
