use crate::app::{card, heading, parse_amount, primary_button, App, Msg};
use crate::model::*;
use crate::theme::*;
use eframe::egui::{self, RichText};

#[derive(Default)]
pub struct FinanceState {
    pub income: bool, // false = expense
    pub amount: String,
    pub category: String,
    pub note: String,
    pub repair: bool,
    pub customer: String,
    pub phone: String,
    pub problem: String,
    pub total: String,
    pub paid: String,
    pub pickup: String,
    pub product: Option<String>,
    pub quantity: String,
    pub search: String,
    pub confirm_delete: Option<String>,
    pub inited: bool,
}

pub fn show(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    if !app.finance.inited {
        app.finance.income = true;
        app.finance.quantity = "1".into();
        app.finance.inited = true;
    }
    heading(ui, "Finance");
    form(app, ctx, ui);
    ui.add_space(14.0);
    list(app, ctx, ui);
}

fn form(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let business = app.mode == "business";
    let products = app.products.clone();
    let mut submit = false;
    card(ui, |ui| {
        let f = &mut app.finance;
        ui.horizontal(|ui| {
            ui.selectable_value(&mut f.income, true, "Income");
            ui.selectable_value(&mut f.income, false, "Expense");
            if business && f.income {
                ui.separator();
                ui.selectable_value(&mut f.repair, false, "Sale");
                ui.selectable_value(&mut f.repair, true, "Repair job");
            }
        });
        let repair = business && f.income && f.repair;
        ui.add_space(4.0);
        if repair {
            ui.columns(2, |c| {
                c[0].label("Customer name");
                c[0].add(egui::TextEdit::singleline(&mut f.customer).desired_width(f32::INFINITY));
                c[1].label("Phone");
                c[1].add(egui::TextEdit::singleline(&mut f.phone).desired_width(f32::INFINITY));
            });
            ui.label("Problem");
            ui.add(egui::TextEdit::singleline(&mut f.problem).desired_width(f32::INFINITY));
            ui.columns(3, |c| {
                c[0].label("Total price");
                c[0].add(egui::TextEdit::singleline(&mut f.total).desired_width(f32::INFINITY));
                c[1].label("Paid now");
                c[1].add(egui::TextEdit::singleline(&mut f.paid).desired_width(f32::INFINITY));
                let due = parse_amount(&f.total).unwrap_or(0.0) - parse_amount(&f.paid).unwrap_or(0.0);
                c[2].label("Still due");
                c[2].label(RichText::new(format!("{:.2}", due.max(0.0))).font(bold_font(16.0)).color(if due > 0.0 { RED } else { GREEN }));
            });
            ui.label("Pickup time (optional)");
            ui.add(egui::TextEdit::singleline(&mut f.pickup).desired_width(f32::INFINITY));
        } else {
            if business && f.income && !products.is_empty() {
                ui.label("Product (optional)");
                let current = f
                    .product
                    .as_ref()
                    .and_then(|id| products.iter().find(|p| &p.id == id))
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "No product".into());
                let mut changed = false;
                egui::ComboBox::from_id_salt("sale-product").selected_text(current).width(300.0).show_ui(ui, |ui| {
                    if ui.selectable_value(&mut f.product, None, "No product").clicked() {
                        changed = true;
                    }
                    for p in &products {
                        if ui.selectable_value(&mut f.product, Some(p.id.clone()), format!("{} ({} in stock)", p.name, p.stock_quantity)).clicked() {
                            changed = true;
                            if f.category.trim().is_empty() {
                                f.category = "Sales".into();
                            }
                        }
                    }
                });
                if f.product.is_some() {
                    ui.label("Quantity");
                    changed |= ui.add(egui::TextEdit::singleline(&mut f.quantity).desired_width(80.0)).changed();
                }
                if changed {
                    if let Some(p) = f.product.as_ref().and_then(|id| products.iter().find(|p| &p.id == id)) {
                        let q = parse_amount(&f.quantity).unwrap_or(1.0);
                        f.amount = format!("{}", p.price * q);
                    }
                }
            }
            ui.columns(2, |c| {
                c[0].label("Amount");
                c[0].add(egui::TextEdit::singleline(&mut f.amount).desired_width(f32::INFINITY));
                c[1].label("Category");
                c[1].add(egui::TextEdit::singleline(&mut f.category).desired_width(f32::INFINITY));
            });
        }
        ui.label("Note (optional)");
        ui.add(egui::TextEdit::singleline(&mut f.note).desired_width(f32::INFINITY));
        ui.add_space(6.0);
        if ui.add(primary_button("Add transaction").min_size(egui::vec2(180.0, 36.0))).clicked() {
            submit = true;
        }
    });
    if submit {
        add(app, ctx);
    }
}

fn add(app: &mut App, ctx: &egui::Context) {
    let business = app.mode == "business";
    let code = app.profile.currency();
    let symbol = app.symbol().to_string();
    let f = &mut app.finance;
    let repair = business && f.income && f.repair;
    let mut tx = NewTransaction {
        kind: if f.income { "income" } else { "expense" }.into(),
        category: f.category.trim().to_string(),
        note: f.note.trim().to_string(),
        currency_code: code,
        currency_symbol: symbol,
        date: chrono::Utc::now().to_rfc3339(),
        sub_type: Some(if repair { "repair" } else { "sale" }.into()),
        ..Default::default()
    };
    let err: Option<&str> = if repair {
        let (total, paid) = (parse_amount(&f.total), parse_amount(&f.paid));
        if f.customer.trim().is_empty() || f.problem.trim().is_empty() || total.is_none() || paid.is_none() {
            Some("Fill in the customer, the problem, the total price and the amount paid.")
        } else if paid.unwrap() < 0.0 || total.unwrap() < 0.0 || paid.unwrap() > total.unwrap() {
            Some("The amount paid cannot be more than the total price.")
        } else {
            let (total, paid) = (total.unwrap(), paid.unwrap());
            tx.amount = paid;
            tx.total_amount = Some(total);
            tx.paid_amount = Some(paid);
            tx.due_amount = Some(total - paid);
            tx.customer_name = Some(f.customer.trim().into());
            tx.customer_phone = Some(f.phone.trim().into());
            tx.problem = Some(f.problem.trim().into());
            tx.pickup_time = Some(f.pickup.trim().into());
            tx.status = Some("pending".into());
            if tx.category.is_empty() {
                tx.category = "Repair".into();
            }
            if tx.note.is_empty() {
                tx.note = format!("{} - {}", f.problem.trim(), f.customer.trim());
            }
            None
        }
    } else {
        match parse_amount(&f.amount) {
            Some(a) if a > 0.0 => {
                if tx.category.is_empty() {
                    Some("Please provide a category.")
                } else {
                    tx.amount = a;
                    if f.income {
                        if let Some(id) = f.product.clone() {
                            match parse_amount(&f.quantity).filter(|q| *q > 0.0 && q.fract() == 0.0) {
                                Some(q) => {
                                    tx.product_id = Some(id);
                                    tx.quantity = Some(q);
                                    None
                                }
                                None => Some("Quantity must be a whole number greater than 0."),
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }
            _ => Some("Enter a valid amount greater than 0."),
        }
    };
    if let Some(e) = err {
        app.say(e, false);
        return;
    }
    if let Some(id) = &tx.product_id {
        tx.product_name = app.products.iter().find(|p| &p.id == id).map(|p| p.name.clone());
    }
    let (owner, mode, who) = (app.owner(), app.mode.clone(), app.profile.name());
    app.finance = FinanceState { income: app.finance.income, repair: app.finance.repair, quantity: "1".into(), inited: true, ..Default::default() };
    app.spawn(ctx, move |c, t| {
        let _ = t.send(Msg::Done(c.add_transaction(&owner, &mode, &who, &tx), "Transaction added."));
    });
}

fn list(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("History").font(bold_font(16.0)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(egui::TextEdit::singleline(&mut app.finance.search).hint_text("Search").desired_width(220.0));
            });
        });
        ui.add_space(6.0);
        let q = app.finance.search.to_lowercase();
        let rows: Vec<Transaction> = app
            .txs
            .iter()
            .filter(|t| {
                q.is_empty()
                    || [&t.category, &t.note, &t.customer_name, &t.product_name]
                        .iter()
                        .any(|f| f.as_deref().map(|s| s.to_lowercase().contains(&q)).unwrap_or(false))
            })
            .take(200)
            .cloned()
            .collect();
        if rows.is_empty() {
            ui.label(RichText::new("Nothing here yet.").color(MUTED));
        }
        let mut delete: Option<String> = None;
        for t in &rows {
            ui.horizontal(|ui| {
                let date = t.day().map(|d| d.format("%d %b %Y").to_string()).unwrap_or_default();
                ui.label(RichText::new(date).color(MUTED).size(13.0));
                ui.label(RichText::new(t.category.clone().unwrap_or_default()).font(bold_font(14.5)));
                if let Some(n) = t.customer_name.as_ref().filter(|n| !n.is_empty()) {
                    ui.label(RichText::new(n).color(MUTED));
                }
                if let Some(due) = t.due_amount.filter(|d| *d > 0.0) {
                    ui.label(RichText::new(format!("due {}", app.money(due))).color(AMBER));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if app.finance.confirm_delete.as_deref() == Some(t.id.as_str()) {
                        if ui.add(egui::Button::new(RichText::new("Delete?").color(egui::Color32::WHITE)).fill(RED)).clicked() {
                            delete = Some(t.id.clone());
                        }
                        if ui.button("Keep").clicked() {
                            app.finance.confirm_delete = None;
                        }
                    } else if ui.small_button("Delete").clicked() {
                        app.finance.confirm_delete = Some(t.id.clone());
                    }
                    let sign = if t.is_income() { "+" } else { "-" };
                    ui.label(RichText::new(format!("{sign}{}", app.money(t.amount))).color(if t.is_income() { GREEN } else { RED }));
                });
            });
            ui.separator();
        }
        if let Some(id) = delete {
            app.finance.confirm_delete = None;
            app.spawn(ctx, move |c, t| {
                let _ = t.send(Msg::Done(c.delete_transaction(&id), "Transaction deleted."));
            });
        }
    });
}
