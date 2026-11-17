//! Inventory: products with stock levels, adding products, and stock changes with their history.

use crate::app::{parse_amount, App, Msg};
use crate::model::*;
use crate::theme::*;
use crate::ui;
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
    pub adding: bool,
    pub selected: Option<String>,
    pub adj_kind: usize, // 0 purchase, 1 adjustment, 2 return
    pub adj_qty: String,
    pub adj_note: String,
    pub search: String,
    pub low_only: bool,
}

const KINDS: [(&str, &str); 3] = [("purchase", "Stock purchase"), ("adjustment", "Adjustment (+/-)"), ("return", "Return")];
const MASK: &str = "••••";

pub fn show(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    if !app.business() {
        ui::card(ui, |ui| {
            ui::empty_state(ui, ic::BOX, "Inventory is part of Business mode", "Switch to Business at the top right to manage products and stock.");
        });
        return;
    }
    let perms = app.perms();
    if !perms.can_view_stock {
        ui::card(ui, |ui| {
            ui::empty_state(ui, ic::LOCK, "No access to stock", "The business owner has not given you permission to see inventory.");
        });
        return;
    }
    let value: f64 = app.products.iter().map(|p| p.stock_quantity * p.cost).sum();
    let low = app.products.iter().filter(|p| p.is_low()).count();
    ui.columns(3, |c| {
        ui::stat(&mut c[0], ic::BOX, "Products", &app.products.len().to_string(), BRAND);
        let v = if perms.can_view_price { app.money(value) } else { MASK.into() };
        ui::stat(&mut c[1], ic::MONEY, "Stock value (cost)", &v, GREEN);
        ui::stat(&mut c[2], ic::WARNING, "Low on stock", &low.to_string(), if low > 0 { RED } else { MUTED });
    });
    ui.add_space(14.0);
    list(app, ctx, ui);
    add_dialog(app, ctx);
    detail_dialog(app, ctx);
}

fn list(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let perms = app.perms();
    let q = app.inventory.search.to_lowercase();
    let low_only = app.inventory.low_only;
    let products: Vec<Product> = app
        .products
        .iter()
        .filter(|p| !low_only || p.is_low())
        .filter(|p| {
            q.is_empty()
                || p.name.to_lowercase().contains(&q)
                || p.sku.as_deref().unwrap_or("").to_lowercase().contains(&q)
                || p.category.as_deref().unwrap_or("").to_lowercase().contains(&q)
        })
        .cloned()
        .collect();
    ui::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Products ({})", products.len())).font(bold_font(16.5)));
            ui.add_space(8.0);
            if ui::chip(ui, "Low stock only", low_only).clicked() {
                app.inventory.low_only = !low_only;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if perms.can_manage_inventory && ui.add(ui::primary_button("+  Add product").min_size(egui::vec2(140.0, 38.0))).clicked() {
                    app.inventory.adding = true;
                }
                ui.add(egui::TextEdit::singleline(&mut app.inventory.search).hint_text("Search name, code, category").desired_width(230.0).margin(egui::Margin::symmetric(10, 7)));
                ui::icon(ui, ic::SEARCH, 14.0, FAINT);
            });
        });
        ui.add_space(8.0);
        if products.is_empty() {
            let (t, s) = if app.products.is_empty() {
                ("No products yet", "Add your first product to start tracking stock.")
            } else {
                ("Nothing matches", "Try another search or turn off the low stock filter.")
            };
            ui::empty_state(ui, ic::BOX, t, s);
        }
        // column titles
        if !products.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(52.0);
                ui.label(RichText::new("PRODUCT").font(bold_font(11.0)).color(FAINT));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(24.0);
                    ui.add_sized([110.0, 16.0], egui::Label::new(RichText::new("STOCK").font(bold_font(11.0)).color(FAINT)));
                    ui.add_sized([110.0, 16.0], egui::Label::new(RichText::new("PRICE").font(bold_font(11.0)).color(FAINT)));
                });
            });
            ui::divider(ui);
        }
        for p in &products {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 58.0), egui::Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(rect, egui::CornerRadius::same(12), SURFACE);
            }
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(egui::vec2(6.0, 0.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
            ui::icon_box(&mut child, ic::BOX, if p.is_low() { RED } else { BRAND }, 38.0);
            child.vertical(|ui| {
                ui.add_space(10.0);
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(RichText::new(&p.name).font(bold_font(14.5)));
                let mut bits = vec![];
                if let Some(s) = p.sku.as_ref().filter(|s| !s.is_empty()) {
                    bits.push(s.clone());
                }
                if let Some(c) = p.category.as_ref().filter(|s| !s.is_empty()) {
                    bits.push(c.clone());
                }
                ui.label(RichText::new(bits.join("  ·  ")).color(MUTED).size(12.5));
            });
            child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui::icon(ui, ic::CHEVRON_RIGHT, 11.0, FAINT);
                ui.allocate_ui_with_layout(egui::vec2(110.0, 30.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let s = format!("{} in stock", p.stock_quantity);
                    if p.is_low() {
                        ui::pill(ui, &s, RED, RED_SOFT);
                    } else {
                        ui::pill(ui, &s, GREEN, GREEN_SOFT);
                    }
                });
                ui.allocate_ui_with_layout(egui::vec2(110.0, 30.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let price = if perms.can_view_price { app.money(p.price) } else { MASK.into() };
                    ui.label(RichText::new(price).font(bold_font(14.0)));
                });
            });
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                app.inventory.selected = Some(p.id.clone());
                app.moves.clear();
                app.load_moves(ctx, p.id.clone());
            }
        }
    });
}

fn add_dialog(app: &mut App, ctx: &egui::Context) {
    if !app.inventory.adding {
        return;
    }
    let mut submit = false;
    let (_, closed) = ui::dialog(ctx, "add-product", "Add a product", 520.0, |ui| {
        let s = &mut app.inventory;
        ui::field(ui, "Name", &mut s.name, "e.g. Phone case");
        ui.columns(2, |c| {
            ui::field(&mut c[0], "SKU / code", &mut s.sku, "Optional");
            ui::field(&mut c[1], "Category", &mut s.category, "Optional");
        });
        ui.columns(2, |c| {
            ui::field(&mut c[0], "Selling price", &mut s.price, "0");
            ui::field(&mut c[1], "Cost price", &mut s.cost, "0");
        });
        ui.columns(2, |c| {
            ui::field(&mut c[0], "Stock now", &mut s.stock, "0");
            ui::field(&mut c[1], "Warn me when stock is at or below", &mut s.min, "0");
        });
        ui.add_space(14.0);
        if ui.add(ui::primary_button("Add product").min_size(egui::vec2(ui.available_width(), 42.0))).clicked() {
            submit = true;
        }
    });
    if closed {
        app.inventory.adding = false;
        return;
    }
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
    let keep = (app.inventory.selected.clone(), app.inventory.adj_kind, std::mem::take(&mut app.inventory.search), app.inventory.low_only);
    app.inventory = InventoryState { selected: keep.0, adj_kind: keep.1, search: keep.2, low_only: keep.3, ..Default::default() };
    app.spawn(ctx, move |c, t| {
        let _ = t.send(Msg::Done(c.add_product(&owner, &uid, &who, &p), "Product added."));
    });
}

fn detail_dialog(app: &mut App, ctx: &egui::Context) {
    let Some(p) = app.inventory.selected.as_ref().and_then(|id| app.products.iter().find(|p| &p.id == id)).cloned() else {
        return;
    };
    let perms = app.perms();
    let mut submit = false;
    let moves = app.moves.clone();
    let price = if perms.can_view_price { app.money(p.price) } else { MASK.into() };
    let cost = if perms.can_view_price { app.money(p.cost) } else { MASK.into() };
    let (_, closed) = ui::dialog(ctx, "product-detail", &p.name, 560.0, |ui| {
        ui.horizontal(|ui| {
            for (label, v) in [("In stock", p.stock_quantity.to_string()), ("Warn at", p.min_stock_level.to_string()), ("Price", price.clone()), ("Cost", cost.clone())] {
                egui::Frame::new().fill(SURFACE).corner_radius(egui::CornerRadius::same(12)).inner_margin(egui::Margin::symmetric(12, 8)).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(RichText::new(label).color(MUTED).size(12.0));
                        ui.label(RichText::new(v).font(bold_font(16.0)));
                    });
                });
            }
        });
        if p.is_low() {
            ui.add_space(6.0);
            ui::pill(ui, "Low on stock", RED, RED_SOFT);
        }
        if perms.can_manage_inventory {
            ui.add_space(12.0);
            ui.label(RichText::new("Change stock").font(bold_font(14.5)));
            let s = &mut app.inventory;
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("adj-kind").selected_text(KINDS[s.adj_kind].1).width(170.0).show_ui(ui, |ui| {
                    for (i, (_, label)) in KINDS.iter().enumerate() {
                        ui.selectable_value(&mut s.adj_kind, i, *label);
                    }
                });
                ui.add(egui::TextEdit::singleline(&mut s.adj_qty).hint_text("Quantity").desired_width(90.0).margin(egui::Margin::symmetric(10, 7)));
                ui.add(egui::TextEdit::singleline(&mut s.adj_note).hint_text("Note (optional)").desired_width(170.0).margin(egui::Margin::symmetric(10, 7)));
                if ui.add(ui::primary_button("Save").min_size(egui::vec2(70.0, 36.0))).clicked() {
                    submit = true;
                }
            });
        }
        ui.add_space(12.0);
        ui.label(RichText::new("Stock history").font(bold_font(14.5)));
        ui.add_space(4.0);
        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            if moves.is_empty() {
                ui.label(RichText::new("No stock changes yet.").color(MUTED));
            }
            for (i, m) in moves.iter().take(50).enumerate() {
                if i > 0 {
                    ui::divider(ui);
                }
                ui.horizontal(|ui| {
                    ui.set_min_height(38.0);
                    let up = m.quantity >= 0.0;
                    ui::icon_circle(ui, if up { ic::UP } else { ic::DOWN }, if up { GREEN } else { RED }, 28.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        let kind = match m.kind.as_str() {
                            "purchase" => "Stock purchase",
                            "sale" => "Sale",
                            "adjustment" => "Adjustment",
                            "return" => "Return",
                            k => k,
                        };
                        ui.label(RichText::new(kind).font(bold_font(13.5)));
                        let date = chrono::DateTime::parse_from_rfc3339(&m.date)
                            .map(|d| d.with_timezone(&chrono::Local).format("%d %b %Y").to_string())
                            .unwrap_or_default();
                        let note = m.note.clone().filter(|n| !n.is_empty()).map(|n| format!("  ·  {n}")).unwrap_or_default();
                        ui.label(RichText::new(format!("{date}{note}")).color(MUTED).size(12.0));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{:+}", m.quantity)).font(bold_font(14.0)).color(if up { GREEN } else { RED }));
                    });
                });
            }
        });
    });
    if closed {
        app.inventory.selected = None;
        app.moves_for = None;
        return;
    }
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
