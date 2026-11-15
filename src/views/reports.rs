use crate::app::{card, heading, stat, App};
use crate::theme::*;
use chrono::{Datelike, Duration, Local, NaiveDate};
use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke};

#[derive(PartialEq, Clone, Copy, Default)]
pub enum Period {
    #[default]
    Week,
    Month,
    Year,
}

#[derive(Default)]
pub struct ReportsState {
    pub period: Period,
}

/// Start of the period, and the buckets (label, first day, last day inclusive) to chart.
fn buckets(period: Period, today: NaiveDate) -> Vec<(String, NaiveDate, NaiveDate)> {
    match period {
        Period::Week => (0..7)
            .rev()
            .map(|i| {
                let d = today - Duration::days(i);
                (d.format("%a").to_string(), d, d)
            })
            .collect(),
        Period::Month => (0..30)
            .rev()
            .map(|i| {
                let d = today - Duration::days(i);
                (d.format("%d").to_string(), d, d)
            })
            .collect(),
        Period::Year => (0..12)
            .rev()
            .map(|i| {
                let total = today.year() * 12 + today.month0() as i32 - i;
                let (y, m) = (total.div_euclid(12), total.rem_euclid(12) as u32 + 1);
                let first = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
                let next = if m == 12 { NaiveDate::from_ymd_opt(y + 1, 1, 1) } else { NaiveDate::from_ymd_opt(y, m + 1, 1) }.unwrap();
                (first.format("%b").to_string(), first, next - Duration::days(1))
            })
            .collect(),
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    heading(ui, "Reports");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut app.reports.period, Period::Week, "Last 7 days");
        ui.selectable_value(&mut app.reports.period, Period::Month, "Last 30 days");
        ui.selectable_value(&mut app.reports.period, Period::Year, "Last 12 months");
    });
    ui.add_space(8.0);

    let today = Local::now().date_naive();
    let bs = buckets(app.reports.period, today);
    let (start, end) = (bs.first().unwrap().1, bs.last().unwrap().2);
    let mut data: Vec<(f64, f64)> = vec![(0.0, 0.0); bs.len()];
    let mut categories: std::collections::BTreeMap<String, f64> = Default::default();
    for t in &app.txs {
        let Some(d) = t.day() else { continue };
        if d < start || d > end {
            continue;
        }
        if let Some(i) = bs.iter().position(|(_, a, b)| d >= *a && d <= *b) {
            if t.is_income() {
                data[i].0 += t.amount;
            } else {
                data[i].1 += t.amount;
                *categories.entry(t.category.clone().unwrap_or_else(|| "Uncategorised".into())).or_default() += t.amount;
            }
        }
    }
    let income: f64 = data.iter().map(|d| d.0).sum();
    let expense: f64 = data.iter().map(|d| d.1).sum();

    ui.columns(3, |c| {
        stat(&mut c[0], "Income", &app.money(income), GREEN);
        stat(&mut c[1], "Expenses", &app.money(expense), RED);
        stat(&mut c[2], "Profit", &app.money(income - expense), if income >= expense { BRAND } else { RED });
    });
    ui.add_space(12.0);

    card(ui, |ui| {
        ui.label(RichText::new("Income and expenses").font(bold_font(16.0)));
        ui.horizontal(|ui| {
            ui.colored_label(GREEN, "■ Income");
            ui.colored_label(RED, "■ Expenses");
        });
        let max = data.iter().flat_map(|d| [d.0, d.1]).fold(0.0_f64, f64::max).max(1.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 220.0), Sense::hover());
        let painter = ui.painter_at(rect);
        let plot = rect.shrink2(egui::vec2(4.0, 0.0)).with_max_y(rect.max.y - 20.0);
        painter.line_segment([plot.left_bottom(), plot.right_bottom()], Stroke::new(1.0_f32, LINE_STRONG));
        let n = bs.len() as f32;
        let slot = plot.width() / n;
        let bar = (slot * 0.34).clamp(2.0, 22.0);
        let show_every = if bs.len() > 12 { 3 } else { 1 };
        for (i, ((label, _, _), (inc, exp))) in bs.iter().zip(&data).enumerate() {
            let cx = plot.left() + slot * (i as f32 + 0.5);
            for (v, color, dx) in [(*inc, GREEN, -bar / 2.0), (*exp, RED, bar / 2.0)] {
                let h = (v / max) as f32 * (plot.height() - 6.0);
                if h > 0.0 {
                    let r = egui::Rect::from_min_max(egui::pos2(cx + dx - bar / 2.0, plot.bottom() - h), egui::pos2(cx + dx + bar / 2.0, plot.bottom()));
                    painter.rect_filled(r, CornerRadius::same(3), color);
                }
            }
            if i % show_every == 0 {
                painter.text(egui::pos2(cx, plot.bottom() + 4.0), egui::Align2::CENTER_TOP, label, text_font(11.5), MUTED);
            }
        }
    });
    ui.add_space(12.0);

    card(ui, |ui| {
        ui.label(RichText::new("Where the money went").font(bold_font(16.0)));
        ui.add_space(4.0);
        let mut rows: Vec<_> = categories.into_iter().collect();
        rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if rows.is_empty() {
            ui.label(RichText::new("No expenses in this period.").color(MUTED));
        }
        for (name, amount) in rows.iter().take(8) {
            let share = if expense > 0.0 { (*amount / expense) as f32 } else { 0.0 };
            ui.horizontal(|ui| {
                ui.label(name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(app.money(*amount)).color(MUTED));
                });
            });
            let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), Sense::hover());
            ui.painter().rect_filled(r, CornerRadius::same(3), SURFACE_MUTED);
            ui.painter().rect_filled(r.with_max_x(r.left() + r.width() * share), CornerRadius::same(3), Color32::from_rgb(0xF0, 0x8A, 0x8A));
            ui.add_space(4.0);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_buckets_cover_twelve_months_ending_this_month() {
        let today = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let b = buckets(Period::Year, today);
        assert_eq!(b.len(), 12);
        assert_eq!(b[0].1, NaiveDate::from_ymd_opt(2025, 4, 1).unwrap());
        assert_eq!(b[11].1, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
        assert_eq!(b[11].2, NaiveDate::from_ymd_opt(2026, 3, 31).unwrap());
    }

    #[test]
    fn week_buckets_end_today() {
        let today = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let b = buckets(Period::Week, today);
        assert_eq!(b.len(), 7);
        assert_eq!(b[6].1, today);
    }
}
