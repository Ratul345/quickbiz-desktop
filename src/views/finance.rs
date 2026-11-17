//! Finance: add income and expenses (including repair jobs with money still due) and browse the history.

use crate::app::{parse_amount, App, Msg};
use crate::model::*;
use crate::theme::*;
use crate::ui;
use chrono::{Datelike, Local};
use eframe::egui::{self, CornerRadius, RichText};

#[derive(PartialEq, Clone, Copy, Default)]
pub enum Period {
    Today,
    #[default]
    Month,
    All,
}

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
    pub period: Period,
    pub kind_filter: u8, // 0 all, 1 income, 2 expense
    pub confirm_delete: Option<String>,
    pub done: bool,
    pub order_type: String,
    pub items: String,
    pub pay_for: Option<Transaction>,
    pub pay_input: String,
    pub focus: bool,
    pub inited: bool,
}

pub fn show(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    if !app.finance.inited {
        app.finance.income = true;
        app.finance.inited = true;
    }
    if app.finance.quantity.is_empty() {
        app.finance.quantity = "1".into();
    }
    let can_add = app.perms().can_manage_transactions;
    if ui.available_width() > 980.0 {
        let total = ui.available_width();
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(egui::vec2(370.0, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.set_width(370.0);
                if can_add {
                    form(app, ctx, ui)
                } else {
                    no_access(ui)
                }
            });
            ui.add_space(6.0);
            ui.allocate_ui_with_layout(egui::vec2(total - 386.0, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.set_width(total - 386.0);
                list(app, ui);
            });
        });
    } else {
        if can_add {
            form(app, ctx, ui)
        } else {
            no_access(ui)
        }
        ui.add_space(14.0);
        list(app, ui);
    }
    delete_dialog(app, ctx);
    payment_dialog(app, ctx);
}

fn no_access(ui: &mut egui::Ui) {
    ui::card(ui, |ui| {
        ui::empty_state(ui, ic::LOCK, "View only", "The business owner has not given you permission to record sales or expenses.");
    });
}

fn form(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let business = app.mode == "business";
    let special = special_for(app.profile.business_category.as_deref());
    let biz_cat = app.profile.business_category.clone().unwrap_or_else(|| "Other".into());
    let products = app.products.clone();
    let mut submit = false;
    let focus = std::mem::take(&mut app.finance.focus);
    ui::card(ui, |ui| {
        let f = &mut app.finance;
        ui.label(RichText::new("New entry").font(bold_font(16.5)));
        ui.add_space(8.0);
        let mut kind = if f.income { 0 } else { 1 };
        if ui::segmented(ui, &mut kind, &[(0, "Income"), (1, "Expense")]) {
            f.income = kind == 0;
        }
        if business && f.income {
            ui.add_space(6.0);
            let mut sub = if f.repair { 1 } else { 0 };
            if ui::segmented(ui, &mut sub, &[(0, special.sale_label), (1, special.label)]) {
                f.repair = sub == 1;
            }
        }
        let kind = if business && f.income && f.repair { special.kind } else { Kind::Sale };
        ui.add_space(8.0);
        if kind.has_due() {
            let job = kind == Kind::Job;
            ui::field(ui, if job { "Client name" } else { "Customer name" }, &mut f.customer, "e.g. Rahim");
            ui::field(ui, "Phone", &mut f.phone, if kind == Kind::Repair { "" } else { "Optional" });
            match kind {
                Kind::Repair => {
                    ui::field(ui, "Problem", &mut f.problem, "e.g. Broken screen");
                }
                Kind::Job => {
                    ui::field(ui, "Work", &mut f.problem, "e.g. Website design, AC servicing");
                }
                _ => {}
            }
            ui.columns(2, |c| {
                ui::field(&mut c[0], "Total", &mut f.total, "0");
                ui::field(&mut c[1], "Paid now", &mut f.paid, "0");
            });
            let due = parse_amount(&f.total).unwrap_or(0.0) - parse_amount(&f.paid).unwrap_or(0.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Still due").color(MUTED));
                ui.label(RichText::new(format!("{:.2}", due.max(0.0))).font(bold_font(16.0)).color(if due > 0.0 { RED } else { GREEN }));
            });
            match kind {
                Kind::Repair => {
                    ui::field(ui, "Pickup time", &mut f.pickup, "Optional, e.g. Friday 5 pm");
                }
                Kind::Job => {
                    ui::field(ui, "Due date", &mut f.pickup, "Optional, e.g. 30 Nov");
                }
                _ => {}
            }
            if kind != Kind::Credit {
                ui.add_space(4.0);
                let mut done = if f.done { 1 } else { 0 };
                if ui::segmented(ui, &mut done, &[(0, "Pending"), (1, "Completed")]) {
                    f.done = done == 1;
                }
            }
        } else {
            if kind == Kind::Order {
                ui.label(RichText::new("Order type").font(bold_font(13.0)).color(MUTED));
                if f.order_type.is_empty() {
                    f.order_type = ORDER_TYPES[0].into();
                }
                ui.horizontal(|ui| {
                    for o in ORDER_TYPES {
                        if ui::chip(ui, o, f.order_type == o).clicked() {
                            f.order_type = o.into();
                        }
                    }
                });
                if f.order_type == "Dine-in" {
                    ui::field(ui, "Table number", &mut f.problem, "Optional, e.g. 5");
                } else {
                    ui::field(ui, "Customer name", &mut f.customer, "Optional");
                    ui::field(ui, "Phone", &mut f.phone, "Optional");
                }
                ui::field(ui, "Dishes", &mut f.items, "e.g. Biryani x2, Lassi");
            }
            if business && f.income && !products.is_empty() {
                ui.label(RichText::new("Product").font(bold_font(13.0)).color(MUTED));
                let current = f
                    .product
                    .as_ref()
                    .and_then(|id| products.iter().find(|p| &p.id == id))
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "No product".into());
                let mut changed = false;
                egui::ComboBox::from_id_salt("sale-product").selected_text(current).width(ui.available_width()).show_ui(ui, |ui| {
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
                    changed |= ui::field(ui, "Quantity", &mut f.quantity, "1").changed();
                }
                if changed {
                    if let Some(p) = f.product.as_ref().and_then(|id| products.iter().find(|p| &p.id == id)) {
                        let q = parse_amount(&f.quantity).unwrap_or(1.0);
                        f.amount = format!("{}", p.price * q);
                    }
                }
            }
            let a = ui::field(ui, "Amount", &mut f.amount, "0");
            if focus {
                a.request_focus();
            }
            ui::field(ui, "Category", &mut f.category, if f.income { "e.g. Sales" } else { "e.g. Rent, Food" });
            ui.horizontal_wrapped(|ui| {
                for (label, value) in category_presets(business, &biz_cat, f.income) {
                    if ui::chip(ui, label, f.category == value).clicked() {
                        f.category = value.to_string();
                    }
                }
            });
        }
        ui::field(ui, "Note", &mut f.note, "Optional");
        ui.add_space(10.0);
        let label = if f.income { "Add income" } else { "Add expense" };
        if ui.add(ui::primary_button(label).min_size(egui::vec2(ui.available_width(), 42.0))).clicked() {
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
    let special = special_for(app.profile.business_category.as_deref()).kind;
    let f = &mut app.finance;
    let kind = if business && f.income && f.repair { special } else { Kind::Sale };
    let repair = kind.has_due();
    let mut tx = NewTransaction {
        kind: if f.income { "income" } else { "expense" }.into(),
        category: f.category.trim().to_string(),
        note: f.note.trim().to_string(),
        currency_code: code,
        currency_symbol: symbol,
        date: chrono::Utc::now().to_rfc3339(),
        sub_type: Some(kind.code().into()),
        ..Default::default()
    };
    let err: Option<&str> = if repair {
        let (total, paid) = (parse_amount(&f.total), parse_amount(&f.paid));
        let needs_problem = matches!(kind, Kind::Repair | Kind::Job);
        if f.customer.trim().is_empty() || (needs_problem && f.problem.trim().is_empty()) || total.is_none() || paid.is_none() {
            Some(if needs_problem { "Fill in the customer, what the work is, the total and the amount paid." } else { "Fill in the customer, the total and the amount paid." })
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
            if !f.problem.trim().is_empty() {
                tx.problem = Some(f.problem.trim().into());
            }
            tx.pickup_time = Some(f.pickup.trim().into());
            let status = match kind {
                Kind::Credit if total - paid > 0.0 => "pending",
                Kind::Credit => "completed",
                _ if f.done => "completed",
                _ => "pending",
            };
            tx.status = Some(status.into());
            if tx.category.is_empty() {
                tx.category = if kind == Kind::Repair { "Repair Service" } else { "Sales" }.into();
            }
            if tx.note.is_empty() {
                tx.note = if f.problem.trim().is_empty() {
                    format!("{} - {}", kind.label(), f.customer.trim())
                } else {
                    format!("{} - {}", f.problem.trim(), f.customer.trim())
                };
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
    if kind == Kind::Order {
        let f = &app.finance;
        let order = if f.order_type.is_empty() { ORDER_TYPES[0].to_string() } else { f.order_type.clone() };
        tx.items = f.items.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if order == "Dine-in" && !f.problem.trim().is_empty() {
            tx.problem = Some(f.problem.trim().to_string());
        }
        if order != "Dine-in" {
            tx.customer_name = Some(f.customer.trim().to_string()).filter(|s| !s.is_empty());
            tx.customer_phone = Some(f.phone.trim().to_string()).filter(|s| !s.is_empty());
        }
        if tx.note.is_empty() {
            tx.note = match &tx.problem {
                Some(t) => format!("{order} - Table {t}"),
                None => order.clone(),
            };
        }
        tx.description = Some(order);
    }
    if let Some(id) = &tx.product_id {
        tx.product_name = app.products.iter().find(|p| &p.id == id).map(|p| p.name.clone());
    }
    let (owner, mode, who) = (app.owner(), app.mode.clone(), app.profile.name());
    app.finance = FinanceState { income: app.finance.income, repair: app.finance.repair, quantity: "1".into(), period: app.finance.period, kind_filter: app.finance.kind_filter, search: std::mem::take(&mut app.finance.search), inited: true, ..Default::default() };
    app.spawn(ctx, move |c, t| {
        let _ = t.send(Msg::Done(c.add_transaction(&owner, &mode, &who, &tx), "Transaction added."));
    });
}

fn in_period(t: &Transaction, p: Period) -> bool {
    let today = Local::now().date_naive();
    match (p, t.day()) {
        (Period::All, _) => true,
        (_, None) => false,
        (Period::Today, Some(d)) => d == today,
        (Period::Month, Some(d)) => d.year() == today.year() && d.month() == today.month(),
    }
}

fn list(app: &mut App, ui: &mut egui::Ui) {
    let period = app.finance.period;
    let q = app.finance.search.to_lowercase();
    let kf = app.finance.kind_filter;
    let rows: Vec<Transaction> = app
        .txs
        .iter()
        .filter(|t| in_period(t, period))
        .filter(|t| kf == 0 || (kf == 1) == t.is_income())
        .filter(|t| {
            q.is_empty()
                || [&t.category, &t.note, &t.customer_name, &t.product_name, &t.created_by_name]
                    .iter()
                    .any(|f| f.as_deref().map(|s| s.to_lowercase().contains(&q)).unwrap_or(false))
        })
        .cloned()
        .collect();
    let income: f64 = rows.iter().filter(|t| t.is_income()).map(|t| t.amount).sum();
    let expense: f64 = rows.iter().filter(|t| !t.is_income()).map(|t| t.amount).sum();

    ui.horizontal_wrapped(|ui| {
        ui::segmented(ui, &mut app.finance.period, &[(Period::Today, "Today"), (Period::Month, "This month"), (Period::All, "All time")]);
        ui.add_space(6.0);
        ui::segmented(ui, &mut app.finance.kind_filter, &[(0, "All"), (1, "Income"), (2, "Expenses")]);
    });
    ui.add_space(10.0);
    ui.columns(3, |c| {
        ui::stat(&mut c[0], ic::DOWN, "Income", &app.money(income), GREEN);
        ui::stat(&mut c[1], ic::UP, "Expenses", &app.money(expense), RED);
        ui::stat(&mut c[2], ic::MONEY, "Net", &app.money(income - expense), if income >= expense { BRAND } else { RED });
    });
    ui.add_space(12.0);
    let can_delete = app.perms().can_manage_transactions;
    ui::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Transactions ({})", rows.len())).font(bold_font(16.5)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.finance.search)
                        .hint_text("Search category, note, customer")
                        .desired_width(240.0)
                        .margin(egui::Margin::symmetric(10, 7)),
                );
                ui::icon(ui, ic::SEARCH, 14.0, FAINT);
            });
        });
        ui.add_space(6.0);
        if rows.is_empty() {
            ui::empty_state(ui, ic::CARD, "Nothing here", "No transactions match this period or search.");
        }
        for (i, t) in rows.iter().take(300).enumerate() {
            if i > 0 {
                ui::divider(ui);
            }
            match row(app, ui, t, can_delete) {
                RowAction::Delete => app.finance.confirm_delete = Some(t.id.clone()),
                RowAction::Pay => {
                    app.finance.pay_input.clear();
                    app.finance.pay_for = Some(t.clone());
                }
                RowAction::None => {}
            }
        }
        if rows.len() > 300 {
            ui.label(RichText::new("Showing the latest 300. Use search or a shorter period to find older entries.").color(MUTED).size(12.5));
        }
    });
}

/// What the user clicked on a transaction line.
#[derive(PartialEq)]
pub enum RowAction {
    None,
    Delete,
    Pay,
}

/// One transaction line: coloured arrow, title, details, amount, and its buttons.
pub fn row(app: &App, ui: &mut egui::Ui, t: &Transaction, deletable: bool) -> RowAction {
    let mut action = RowAction::None;
    let inc = t.is_income();
    let color = if inc { GREEN } else { RED };
    let kind = t.kind();
    ui.horizontal(|ui| {
        ui.set_min_height(54.0);
        let glyph = match kind {
            Kind::Repair => ic::WRENCH,
            Kind::Credit => ic::CARD,
            Kind::Job => ic::TAG,
            Kind::Order => ic::SHOP,
            Kind::Sale if inc => ic::DOWN,
            Kind::Sale => ic::UP,
        };
        ui::icon_circle(ui, glyph, color, 40.0);
        ui.vertical(|ui| {
            ui.add_space(7.0);
            ui.spacing_mut().item_spacing.y = 1.0;
            let title = t
                .product_name
                .clone()
                .filter(|s| !s.is_empty())
                .or_else(|| t.category.clone().filter(|s| !s.is_empty()))
                .unwrap_or_else(|| "Uncategorised".into());
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).font(bold_font(14.5)));
                if let Some(q) = t.quantity.filter(|q| *q > 0.0) {
                    ui.label(RichText::new(format!("x{q}")).color(MUTED).size(12.5));
                }
                if kind != Kind::Sale {
                    ui::pill(ui, kind.label(), BRAND, BRAND_SOFT);
                }
                if let Some(due) = t.due_amount.filter(|d| *d > 0.0) {
                    ui::pill(ui, &format!("Due {}", app.money(due)), AMBER, AMBER_SOFT);
                }
            });
            let mut bits: Vec<String> = vec![];
            if let Some(n) = t.customer_name.as_ref().filter(|n| !n.is_empty()) {
                bits.push(n.clone());
            }
            if let Some(n) = t.note.as_ref().filter(|n| !n.is_empty()) {
                bits.push(n.chars().take(60).collect());
            }
            if let Some(d) = t.day() {
                bits.push(d.format("%d %b %Y").to_string());
            }
            if let Some(by) = t.created_by_name.as_ref().filter(|n| !n.is_empty()) {
                bits.push(format!("by {by}"));
            }
            ui.label(RichText::new(bits.join("  ·  ")).color(MUTED).size(12.5));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if deletable {
                let b = egui::Button::new(ui::icon_text(ic::DELETE, 13.0, FAINT))
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(CornerRadius::same(8))
                    .min_size(egui::vec2(30.0, 30.0));
                if ui.add(b).on_hover_text("Delete").clicked() {
                    action = RowAction::Delete;
                }
                if t.due_amount.unwrap_or(0.0) > 0.0 && ui.add(ui::secondary_button("Record payment").min_size(egui::vec2(0.0, 30.0))).clicked() {
                    action = RowAction::Pay;
                }
            }
            let sign = if inc { "+" } else { "-" };
            ui.label(RichText::new(format!("{sign}{}", app.money(t.amount))).font(bold_font(15.0)).color(color));
        });
    });
    action
}

fn delete_dialog(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.finance.confirm_delete.clone() else { return };
    let text = "This removes the transaction. If it sold a product, the stock goes back up. This cannot be undone.";
    match ui::confirm(ctx, "delete-tx", "Delete transaction?", text, "Delete", true) {
        Some(true) => {
            app.finance.confirm_delete = None;
            app.spawn(ctx, move |c, t| {
                let _ = t.send(Msg::Done(c.delete_transaction(&id), "Transaction deleted."));
            });
        }
        Some(false) => app.finance.confirm_delete = None,
        None => {}
    }
}

/// Records money a customer pays later on an entry that still has some due.
pub fn payment_dialog(app: &mut App, ctx: &egui::Context) {
    let Some(t) = app.finance.pay_for.clone() else { return };
    let due = t.due_amount.unwrap_or(0.0);
    let mut save = false;
    let who = t.customer_name.clone().filter(|n| !n.is_empty());
    let (_, closed) = ui::dialog(ctx, "record-payment", "Record payment", 420.0, |ui| {
        let text = match &who {
            Some(n) => format!("{n} still owes {}.", app.money(due)),
            None => format!("Still due: {}.", app.money(due)),
        };
        ui.label(RichText::new(text).color(MUTED));
        ui.add_space(8.0);
        ui::field(ui, "Amount received", &mut app.finance.pay_input, "0");
        if ui::link(ui, "Paid in full").clicked() {
            app.finance.pay_input = format!("{due}");
        }
        ui.add_space(12.0);
        if ui.add(ui::primary_button("Save payment").min_size(egui::vec2(ui.available_width(), 42.0))).clicked() {
            save = true;
        }
    });
    if closed {
        app.finance.pay_for = None;
        return;
    }
    if !save {
        return;
    }
    let payment = parse_amount(&app.finance.pay_input).unwrap_or(0.0);
    match apply_payment(&t, payment) {
        Err(e) => app.say(e, false),
        Ok((amount, paid, left, status)) => {
            app.finance.pay_for = None;
            let mut body = serde_json::json!({ "amount": amount, "paidAmount": paid, "dueAmount": left });
            if let Some(s) = status {
                body["status"] = serde_json::json!(s);
            }
            let id = t.id.clone();
            let msg = if left > 0.0 { "Payment recorded." } else { "Payment recorded. This is now fully paid." };
            app.write(ctx, msg, move |c| c.update_transaction(&id, body));
        }
    }
}
