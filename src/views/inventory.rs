use crate::app::{card, heading, parse_amount, primary_button, App, Msg};
use crate::model::*;
use crate::theme::*;
use eframe::egui::{self, RichText};

#[derive(Default)]
pub struct InventoryState {
    pub name: String,
    pub sku: String,
    pub price: String,
    pub cost: String,
    pub stock: String,
    pub min: String,
    pub category: String,
    pub selected: Option<String>,
    pub adj_kind: usize, // 0 purchase, 1 adjustment, 2 return
    pub adj_qty: String,
    pub adj_note: String,
}

const KINDS: [(&str, &str); 3] = [("purchase", "Stock purchase"), ("adjustment", "Adjustment (+/-)"), ("return", "Return")];

pub fn show(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    heading(ui, "Inventory");
    if app.mode != "business" {
        card(ui, |ui| {
            ui.label("Inventory is part of Business mode. Switch to Business at the top.");
        });
        return;
    }
    add_form(app, ctx, ui);
    ui.add_space(14.0);
    table(app, ctx, ui);
    if app.inventory.selected.is_some() {
        ui.add_space(14.0);
        detail(app, ctx, ui);
    }
}

fn add_form(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let mut submit = false;
    card(ui, |ui| {
        ui.label(RichText::new("Add a product").font(bold_font(16.0)));
        let s = &mut app.inventory;
        ui.columns(3, |c| {
            c[0].label("Name");
            c[0].add(egui::TextEdit::singleline(&mut s.name).desired_width(f32::INFINITY));
            c[1].label("SKU / code");
            c[1].add(egui::TextEdit::singleline(&mut s.sku).desired_width(f32::INFINITY));
            c[2].label("Category");
            c[2].add(egui::TextEdit::singleline(&mut s.category).desired_width(f32::INFINITY));
        });
        ui.columns(4, |c| {
            c[0].label("Selling price");
            c[0].add(egui::TextEdit::singleline(&mut s.price).desired_width(f32::INFINITY));
            c[1].label("Cost price");
            c[1].add(egui::TextEdit::singleline(&mut s.cost).desired_width(f32::INFINITY));
            c[2].label("Stock now");
            c[2].add(egui::TextEdit::singleline(&mut s.stock).desired_width(f32::INFINITY));
            c[3].label("Low-stock level");
            c[3].add(egui::TextEdit::singleline(&mut s.min).desired_width(f32::INFINITY));
        });
        ui.add_space(6.0);
        if ui.add(primary_button("Add product").min_size(egui::vec2(160.0, 36.0))).clicked() {
            submit = true;
        }
    });
    if !submit {
        return;
    }
    let s = &app.inventory;
    let (price, cost) = (parse_amount(&s.price), parse_amount(&s.cost).or(Some(0.0)));
    let (stock, min) = (parse_amount(&s.stock).or(Some(0.0)), parse_amount(&s.min).or(Some(0.0)));
    if s.name.trim().is_empty() {
        app.say("Give the product a name.", false);
        return;
    }
    let (Some(price), Some(cost), Some(stock), Some(min)) = (price, cost, stock, min) else {
        app.say("Prices and stock must be numbers.", false);
        return;
    };
    if price < 0.0 || cost < 0.0 || stock < 0.0 || min < 0.0 {
        app.say("Numbers cannot be negative.", false);
        return;
    }
    let p = NewProduct {
        name: s.name.trim().into(),
        sku: s.sku.trim().into(),
        price,
        cost,
        stock_quantity: stock,
        min_stock_level: min,
        category: s.category.trim().into(),
    };
    let (owner, uid, who) = (app.owner(), app.profile.uid.clone(), app.profile.name());
    let keep = (app.inventory.selected.clone(), app.inventory.adj_kind);
    app.inventory = InventoryState { selected: keep.0, adj_kind: keep.1, ..Default::default() };
    app.spawn(ctx, move |c, t| {
        let _ = t.send(Msg::Done(c.add_product(&owner, &uid, &who, &p), "Product added."));
    });
}

fn table(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    card(ui, |ui| {
        ui.label(RichText::new(format!("Products ({})", app.products.len())).font(bold_font(16.0)));
        ui.add_space(4.0);
        if app.products.is_empty() {
            ui.label(RichText::new("No products yet.").color(MUTED));
        }
        let products = app.products.clone();
        for p in &products {
            ui.horizontal(|ui| {
                let on = app.inventory.selected.as_deref() == Some(p.id.as_str());
                if ui.selectable_label(on, RichText::new(&p.name).font(bold_font(14.5))).clicked() {
                    app.inventory.selected = if on { None } else { Some(p.id.clone()) };
                    if !on {
                        app.load_moves(ctx, p.id.clone());
                    }
                }
                ui.label(RichText::new(p.sku.clone().unwrap_or_default()).color(MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let color = if p.is_low() { AMBER } else { INK };
                    ui.label(RichText::new(format!("{} in stock", p.stock_quantity)).color(color));
                    ui.label(RichText::new(app.money(p.price)).color(MUTED));
                });
            });
            ui.separator();
        }
    });
}

fn detail(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let Some(p) = app.inventory.selected.as_ref().and_then(|id| app.products.iter().find(|p| &p.id == id)).cloned() else {
        return;
    };
    let mut submit = false;
    card(ui, |ui| {
        ui.label(RichText::new(&p.name).font(bold_font(18.0)));
        ui.label(RichText::new(format!("{} in stock, low-stock level {}", p.stock_quantity, p.min_stock_level)).color(MUTED));
        ui.add_space(6.0);
        let s = &mut app.inventory;
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("adj-kind").selected_text(KINDS[s.adj_kind].1).show_ui(ui, |ui| {
                for (i, (_, label)) in KINDS.iter().enumerate() {
                    ui.selectable_value(&mut s.adj_kind, i, *label);
                }
            });
            ui.add(egui::TextEdit::singleline(&mut s.adj_qty).hint_text("Quantity").desired_width(100.0));
            ui.add(egui::TextEdit::singleline(&mut s.adj_note).hint_text("Note (optional)").desired_width(220.0));
            if ui.add(primary_button("Save")).clicked() {
                submit = true;
            }
        });
        ui.add_space(8.0);
        ui.label(RichText::new("Stock history").font(bold_font(14.5)));
        if app.moves.is_empty() {
            ui.label(RichText::new("No movements yet.").color(MUTED));
        }
        for m in app.moves.iter().take(30) {
            ui.horizontal(|ui| {
                let date = chrono::DateTime::parse_from_rfc3339(&m.date)
                    .map(|d| d.with_timezone(&chrono::Local).format("%d %b %Y").to_string())
                    .unwrap_or_default();
                ui.label(RichText::new(date).color(MUTED).size(13.0));
                ui.label(&m.kind);
                ui.label(RichText::new(m.note.clone().unwrap_or_default()).color(MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let color = if m.quantity >= 0.0 { GREEN } else { RED };
                    ui.label(RichText::new(format!("{:+}", m.quantity)).color(color));
                });
            });
        }
    });
    if !submit {
        return;
    }
    let Some(mut q) = parse_amount(&app.inventory.adj_qty).filter(|q| *q != 0.0) else {
        app.say("Enter a quantity.", false);
        return;
    };
    let kind = KINDS[app.inventory.adj_kind].0;
    if kind != "adjustment" {
        q = q.abs(); // purchases and returns always add stock; only adjustments may be negative
    }
    if kind == "adjustment" && p.stock_quantity + q < 0.0 {
        app.say("That would take the stock below zero.", false);
        return;
    }
    let (note, who) = (app.inventory.adj_note.trim().to_string(), app.profile.name());
    app.inventory.adj_qty.clear();
    app.inventory.adj_note.clear();
    app.spawn(ctx, move |c, t| {
        let _ = t.send(Msg::Done(c.adjust_stock(&p.id, kind, q, &note, &who), "Stock updated."));
    });
}
