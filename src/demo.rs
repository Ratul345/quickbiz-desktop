//! Screenshot mode, compiled only with `--features demo` (never in the real app).
//! It fills the real screens with the made-up data of a pretend shop and saves one screenshot of
//! each screen, so Microsoft Store screenshots show the real app without needing an account.
//!
//!   cargo run --features demo        (writes dist/screenshots/*.png, then quits)

use crate::app::{App, Stage, Tab};
use crate::model::*;
use crate::views::reports::Period;
use chrono::{Duration, Utc};
use eframe::egui;
use std::path::PathBuf;

const SHOTS: [&str; 9] = ["01-sign-in", "02-dashboard", "03-finance", "04-inventory", "05-reports", "06-profile", "07-team", "08-sidebar-folded", "09-support"];
/// Frames to let a screen settle before capturing it.
const SETTLE: u32 = 30;

#[derive(Default)]
pub struct Driver {
    step: usize,
    frame: u32,
    waiting: bool,
}

fn rng(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed >> 33
}

fn day(days_ago: i64, hour: i64) -> String {
    (Utc::now() - Duration::days(days_ago) - Duration::hours(hour)).to_rfc3339()
}

fn products() -> Vec<Product> {
    // (name, sku, price, cost, stock, low-stock level, category)
    let rows: [(&str, &str, f64, f64, f64, f64, &str); 10] = [
        ("Phone case", "CASE-01", 350.0, 150.0, 42.0, 10.0, "Accessories"),
        ("Screen protector", "GLASS-02", 250.0, 90.0, 6.0, 12.0, "Accessories"),
        ("USB-C cable 1m", "CABLE-03", 300.0, 120.0, 28.0, 8.0, "Cables"),
        ("Fast charger 20W", "CHG-04", 1200.0, 750.0, 4.0, 5.0, "Chargers"),
        ("Wireless earbuds", "BUDS-05", 2200.0, 1500.0, 15.0, 5.0, "Audio"),
        ("Power bank 10000mAh", "PWR-06", 1800.0, 1250.0, 9.0, 4.0, "Chargers"),
        ("Memory card 64GB", "MEM-07", 900.0, 600.0, 3.0, 5.0, "Storage"),
        ("Phone stand", "STAND-08", 450.0, 200.0, 19.0, 6.0, "Accessories"),
        ("Car charger", "CAR-09", 650.0, 380.0, 12.0, 4.0, "Chargers"),
        ("Headphones", "HEAD-10", 1500.0, 950.0, 8.0, 3.0, "Audio"),
    ];
    rows.iter()
        .enumerate()
        .map(|(i, r)| Product {
            id: format!("demo-p{i}"),
            name: r.0.into(),
            sku: Some(r.1.into()),
            price: r.2,
            cost: r.3,
            stock_quantity: r.4,
            min_stock_level: r.5,
            category: Some(r.6.into()),
        })
        .collect()
}

fn transactions() -> Vec<Transaction> {
    let mut seed = 7_u64;
    let customers = ["Sample Customer A", "Sample Customer B", "Sample Customer C", "Sample Customer D"];
    let problems = ["Screen replacement", "Battery change", "Charging port repair", "Water damage check"];
    let mut out = Vec::new();
    for d in 0..365_i64 {
        // a few sales most days, busier lately
        let sales = 1 + (rng(&mut seed) % 3) as i64 - if d > 200 { 1 } else { 0 };
        for s in 0..sales.max(0) {
            let amount = 300.0 + (rng(&mut seed) % 30) as f64 * 100.0;
            out.push(Transaction {
                id: format!("demo-s{d}-{s}"),
                kind: "income".into(),
                amount,
                category: Some("Sales".into()),
                date: day(d, 9 + s * 2),
                sub_type: Some("sale".into()),
                ..Default::default()
            });
        }
        if d % 4 == 0 {
            let total = 1500.0 + (rng(&mut seed) % 40) as f64 * 100.0;
            let paid = if d % 8 == 0 { total } else { (total * 0.5).round() };
            let i = (rng(&mut seed) % 4) as usize;
            out.push(Transaction {
                id: format!("demo-r{d}"),
                kind: "income".into(),
                amount: paid,
                category: Some("Repair".into()),
                date: day(d, 13),
                sub_type: Some("repair".into()),
                customer_name: Some(customers[i].into()),
                problem: Some(problems[i].into()),
                total_amount: Some(total),
                paid_amount: Some(paid),
                due_amount: Some(total - paid),
                status: Some(if d < 12 && d % 8 != 0 { "pending" } else { "completed" }.into()),
                ..Default::default()
            });
        }
        let mut cost = |name: &str, amount: f64, hour: i64| {
            out.push(Transaction {
                id: format!("demo-e-{name}-{d}"),
                kind: "expense".into(),
                amount,
                category: Some(name.into()),
                date: day(d, hour),
                ..Default::default()
            });
        };
        if d % 30 == 3 {
            cost("Rent", 15000.0, 10);
            cost("Salary", 22000.0, 11);
        }
        if d % 30 == 9 {
            cost("Electricity", 3200.0, 12);
        }
        if d % 7 == 2 {
            cost("Stock purchase", 6000.0 + (rng(&mut seed) % 20) as f64 * 250.0, 15);
        }
        if d % 11 == 5 {
            cost("Transport", 600.0 + (rng(&mut seed) % 6) as f64 * 100.0, 16);
        }
    }
    out.sort_by(|a, b| b.date.cmp(&a.date));
    out
}

fn movements() -> Vec<StockMovement> {
    [("purchase", 30.0, "Restock from supplier", 21), ("sale", -2.0, "", 14), ("sale", -3.0, "", 9), ("adjustment", -1.0, "Damaged item", 6), ("sale", -2.0, "", 2), ("return", 1.0, "Customer return", 1)]
        .iter()
        .enumerate()
        .map(|(i, m)| StockMovement {
            id: format!("demo-m{i}"),
            kind: m.0.into(),
            quantity: m.1,
            note: if m.2.is_empty() { None } else { Some(m.2.into()) },
            date: day(m.3, 10),
            created_by_name: Some("Demo Owner".into()),
        })
        .rev()
        .collect()
}

/// Fills the app with the pretend shop (no network, no account).
pub fn load(app: &mut App) {
    app.profile = Profile {
        uid: "demo-owner".into(),
        email: Some("owner@example.com".into()),
        display_name: Some("Demo Owner".into()),
        business_name: Some("Demo Mobile Shop".into()),
        business_id: Some("demo-owner".into()),
        business_owner_id: Some("demo-owner".into()),
        business_category: Some("Mobile Repair".into()),
        role: Some("owner".into()),
        tier: Some("gold".into()),
        is_verified: Some(true),
        currency_code: Some("BDT".into()),
        ..Default::default()
    };
    app.employees = vec![
        Employee { uid: "demo-e1".into(), email: Some("staff.one@example.com".into()), display_name: Some("Staff One".into()), photo_url: None, permissions: Some(Permissions::default()) },
        Employee {
            uid: "demo-e2".into(),
            email: Some("staff.two@example.com".into()),
            display_name: Some("Staff Two".into()),
            photo_url: None,
            permissions: Some(Permissions { can_manage_transactions: true, can_view_reports: true, ..Default::default() }),
        },
    ];
    app.tickets = vec![Ticket {
        id: "demo-t1".into(),
        subject: "[Feature request] Print receipts".into(),
        message: "It would help to print a receipt for each sale.".into(),
        reply: Some("Thanks for the idea! We have added it to our list.".into()),
        status: Some("replied".into()),
        created_at: Some(day(3, 2)),
    }];
    app.mode = "business".into();
    app.txs = transactions();
    app.products = products();
    app.stage = Stage::Login;
}

fn shots_dir() -> PathBuf {
    let dir = std::env::var_os("QB_SHOTS").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("dist").join("screenshots"));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn setup(app: &mut App, step: usize) {
    match step {
        0 => app.stage = Stage::Login,
        1 => {
            app.stage = Stage::Main;
            app.tab = Tab::Dashboard;
        }
        2 => {
            app.tab = Tab::Finance;
            let f = &mut app.finance;
            f.inited = true;
            f.income = true;
            f.repair = true;
            f.quantity = "1".into();
            f.customer = "Sample Customer A".into();
            f.phone = "01700 000000".into();
            f.problem = "Screen replacement".into();
            f.total = "4500".into();
            f.paid = "2000".into();
        }
        3 => {
            app.tab = Tab::Inventory;
            let id = app.products[1].id.clone();
            app.inventory.selected = Some(id.clone());
            app.moves_for = Some(id);
            app.moves = movements();
        }
        4 => {
            app.inventory.selected = None;
            app.tab = Tab::Reports;
            app.reports.period = Period::Month;
        }
        5 => app.tab = Tab::Profile,
        6 => app.tab = Tab::Team,
        7 => {
            app.tab = Tab::Dashboard;
            app.prefs.sidebar_collapsed = true;
        }
        _ => {
            app.prefs.sidebar_collapsed = false;
            app.tab = Tab::Support;
            app.support.tab = 1;
        }
    }
}

/// Called every frame: shows each screen in turn, captures it, and quits when done.
pub fn drive(app: &mut App, ctx: &egui::Context) {
    ctx.request_repaint();

    let arrived: Vec<_> = ctx.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
            .collect()
    });
    for img in arrived {
        let [w, h] = img.size;
        let path = shots_dir().join(format!("{}.png", SHOTS[app.demo.step]));
        if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, img.as_raw().to_vec()) {
            match buf.save(&path) {
                Ok(()) => println!("saved {} ({w}x{h})", path.display()),
                Err(e) => eprintln!("could not save {}: {e}", path.display()),
            }
        }
        app.demo.step += 1;
        app.demo.frame = 0;
        app.demo.waiting = false;
    }

    if app.demo.step >= SHOTS.len() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        return;
    }
    app.demo.frame += 1;
    if app.demo.frame == 1 {
        let step = app.demo.step;
        setup(app, step);
    } else if app.demo.frame >= SETTLE && !app.demo.waiting {
        app.demo.waiting = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
    }
}
