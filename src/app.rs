//! App shell: sign-in state, background loading, tabs, and shared widgets.
//! Network calls run on worker threads and report back through a channel, so the window never freezes.

use crate::api::{Client, Res, Session};
use crate::config;
use crate::model::*;
use crate::store;
use crate::theme::*;
use crate::views;
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub enum Msg {
    Restored(Res<Session>),
    SignedIn(Res<Session>),
    SignedUp(Res<Option<Session>>),
    Profile(Res<Profile>),
    Settings(Res<Settings>),
    Txs(Res<Vec<Transaction>>),
    Products(Res<Vec<Product>>),
    Moves(String, Res<Vec<StockMovement>>),
    /// A write finished. The text is what to tell the user on success.
    Done(Res<()>, &'static str),
}

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Dashboard,
    Finance,
    Inventory,
    Reports,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Stage {
    Booting,
    Login,
    Main,
}

pub struct App {
    pub client: Arc<Client>,
    pub tx: Sender<Msg>,
    rx: Receiver<Msg>,
    pub stage: Stage,
    pub tab: Tab,
    pub mode: String, // "personal" | "business"
    pub session: Option<Session>,
    pub profile: Profile,
    pub settings: Settings,
    pub txs: Vec<Transaction>,
    pub products: Vec<Product>,
    pub moves: Vec<StockMovement>,
    pub moves_for: Option<String>,
    pub loading: u32,
    pub toast: Option<(String, bool, Instant)>,
    pub last_refresh: Instant,
    pub login: views::login::LoginState,
    pub finance: views::finance::FinanceState,
    pub inventory: views::inventory::InventoryState,
    pub reports: views::reports::ReportsState,
    #[cfg(feature = "demo")]
    pub demo: crate::demo::Driver,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply(&cc.egui_ctx);
        crate::logo::install_texture(&cc.egui_ctx);
        let (tx, rx) = channel();
        let client = Arc::new(Client::new());
        let mut app = App {
            client,
            tx,
            rx,
            stage: Stage::Login,
            tab: Tab::Dashboard,
            mode: "business".into(),
            session: None,
            profile: Profile::default(),
            settings: Settings::default(),
            txs: vec![],
            products: vec![],
            moves: vec![],
            moves_for: None,
            loading: 0,
            toast: None,
            last_refresh: Instant::now(),
            login: Default::default(),
            finance: Default::default(),
            inventory: Default::default(),
            reports: Default::default(),
            #[cfg(feature = "demo")]
            demo: Default::default(),
        };
        #[cfg(feature = "demo")]
        crate::demo::load(&mut app);
        #[cfg(not(feature = "demo"))]
        {
            if let Some(saved) = store::load() {
                app.stage = Stage::Booting;
                app.client.set_remember(true);
                app.login.remember = true;
                app.spawn(&cc.egui_ctx, move |c, tx| {
                    let _ = tx.send(Msg::Restored(c.restore(&saved)));
                });
            }
        }
        app
    }

    /// Runs `f` on a worker thread, wakes the UI when it sends something back.
    pub fn spawn<F>(&mut self, ctx: &egui::Context, f: F)
    where
        F: FnOnce(&Client, &Sender<Msg>) + Send + 'static,
    {
        let (client, tx, ctx) = (self.client.clone(), self.tx.clone(), ctx.clone());
        self.loading += 1;
        std::thread::spawn(move || {
            f(&client, &tx);
            ctx.request_repaint();
        });
    }

    pub fn say(&mut self, text: impl Into<String>, ok: bool) {
        self.toast = Some((text.into(), ok, Instant::now()));
    }

    /// Owner whose data we read and write. Personal mode is always your own; business staff use the owner's.
    pub fn owner(&self) -> String {
        if self.mode == "personal" {
            self.profile.uid.clone()
        } else {
            self.profile.data_owner()
        }
    }

    pub fn symbol(&self) -> &'static str {
        currency_symbol(&self.profile.currency())
    }

    pub fn money(&self, v: f64) -> String {
        let neg = v < 0.0;
        let whole = format!("{:.2}", v.abs());
        let (int, frac) = whole.split_once('.').unwrap_or((&whole, "00"));
        let mut grouped = String::new();
        for (i, ch) in int.chars().enumerate() {
            if i > 0 && (int.len() - i) % 3 == 0 {
                grouped.push(',');
            }
            grouped.push(ch);
        }
        let frac = if frac == "00" { String::new() } else { format!(".{frac}") };
        format!("{}{}{}{}", if neg { "-" } else { "" }, self.symbol(), grouped, frac)
    }

    pub fn reload(&mut self, ctx: &egui::Context) {
        self.last_refresh = Instant::now();
        if cfg!(feature = "demo") {
            return; // screenshot mode never touches the network
        }
        let (owner, mode) = (self.owner(), self.mode.clone());
        let o2 = owner.clone();
        self.spawn(ctx, move |c, tx| {
            let _ = tx.send(Msg::Txs(c.transactions(&owner, &mode)));
        });
        self.spawn(ctx, move |c, tx| {
            let _ = tx.send(Msg::Products(c.products(&o2)));
        });
    }

    pub fn load_moves(&mut self, ctx: &egui::Context, product_id: String) {
        let owner = self.owner();
        self.moves_for = Some(product_id.clone());
        self.spawn(ctx, move |c, tx| {
            let r = c.movements(&owner, &product_id);
            let _ = tx.send(Msg::Moves(product_id, r));
        });
    }

    fn after_sign_in(&mut self, ctx: &egui::Context, s: Session) {
        let uid = s.uid.clone();
        self.session = Some(s);
        self.stage = Stage::Main;
        self.login.busy = false;
        self.spawn(ctx, move |c, tx| {
            let _ = tx.send(Msg::Profile(c.profile(&uid)));
        });
        self.spawn(ctx, |c, tx| {
            let _ = tx.send(Msg::Settings(c.settings()));
        });
    }

    pub fn sign_out(&mut self) {
        self.client.sign_out();
        self.session = None;
        self.profile = Profile::default();
        self.txs.clear();
        self.products.clear();
        self.moves.clear();
        self.stage = Stage::Login;
        self.login.busy = false;
        self.login.password.clear();
    }

    fn pump(&mut self, ctx: &egui::Context) {
        while let Ok(m) = self.rx.try_recv() {
            self.loading = self.loading.saturating_sub(1);
            match m {
                Msg::Restored(Ok(s)) => self.after_sign_in(ctx, s),
                Msg::Restored(Err(_)) => {
                    store::clear();
                    self.stage = Stage::Login;
                }
                Msg::SignedIn(Ok(s)) => self.after_sign_in(ctx, s),
                Msg::SignedIn(Err(e)) => {
                    self.login.busy = false;
                    self.login.error = Some(e);
                }
                Msg::SignedUp(Ok(Some(s))) => self.after_sign_in(ctx, s),
                Msg::SignedUp(Ok(None)) => {
                    self.login.busy = false;
                    self.login.register = false;
                    self.login.error = None;
                    self.login.info = Some("Account created. Check your email to confirm it, then sign in.".into());
                }
                Msg::SignedUp(Err(e)) => {
                    self.login.busy = false;
                    self.login.error = Some(e);
                }
                Msg::Profile(Ok(p)) => {
                    let has_business = p.business_id.as_deref().map(|b| !b.is_empty()).unwrap_or(false)
                        || p.business_owner_id.as_deref().map(|b| !b.is_empty()).unwrap_or(false);
                    self.mode = if has_business { "business".into() } else { "personal".into() };
                    self.profile = p;
                    self.reload(ctx);
                }
                Msg::Profile(Err(e)) => self.say(e, false),
                Msg::Settings(Ok(s)) => self.settings = s,
                Msg::Settings(Err(_)) => {}
                Msg::Txs(Ok(v)) => self.txs = v,
                Msg::Txs(Err(e)) => self.say(e, false),
                Msg::Products(Ok(v)) => self.products = v,
                Msg::Products(Err(e)) => self.say(e, false),
                Msg::Moves(id, Ok(v)) => {
                    if self.moves_for.as_deref() == Some(id.as_str()) {
                        self.moves = v;
                    }
                }
                Msg::Moves(_, Err(e)) => self.say(e, false),
                Msg::Done(Ok(()), ok_text) => {
                    self.say(ok_text, true);
                    self.reload(ctx);
                    if let Some(id) = self.moves_for.clone() {
                        self.load_moves(ctx, id);
                    }
                }
                Msg::Done(Err(e), _) => self.say(e, false),
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(feature = "demo")]
        crate::demo::drive(self, ctx);
        self.pump(ctx);

        if self.stage == Stage::Main && self.last_refresh.elapsed() > Duration::from_secs(30) && self.loading == 0 {
            self.reload(ctx);
        }
        if let Some((_, _, at)) = &self.toast {
            if at.elapsed() > Duration::from_secs(5) {
                self.toast = None;
            }
        }
        ctx.request_repaint_after(Duration::from_secs(1));

        match self.stage {
            Stage::Booting => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                    });
                });
            }
            Stage::Login => views::login::show(self, ctx),
            Stage::Main => self.main_ui(ctx),
        }
        self.toast_ui(ctx);
    }
}

impl App {
    fn main_ui(&mut self, ctx: &egui::Context) {
        let maintenance = self.settings.maintenance_mode.unwrap_or(false);
        let update = self
            .settings
            .latest_version
            .as_deref()
            .filter(|v| is_newer_version(v, config::APP_VERSION))
            .map(|v| v.to_string());

        egui::SidePanel::left("nav")
            .exact_width(220.0)
            .frame(egui::Frame::new().fill(WHITE).inner_margin(Margin::same(16)).stroke(Stroke::new(1.0_f32, LINE)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(tex) = crate::logo::texture(ctx) {
                        ui.image((tex.id(), egui::vec2(32.0, 32.0)));
                    }
                    ui.label(RichText::new("QuickBiz").font(bold_font(20.0)).color(BRAND));
                });
                ui.add_space(18.0);
                for (tab, label) in [
                    (Tab::Dashboard, "Dashboard"),
                    (Tab::Finance, "Finance"),
                    (Tab::Inventory, "Inventory"),
                    (Tab::Reports, "Reports"),
                ] {
                    let on = self.tab == tab;
                    let text = RichText::new(label).font(if on { bold_font(15.0) } else { text_font(15.0) }).color(if on { BRAND } else { INK });
                    let btn = egui::Button::new(text)
                        .fill(if on { BRAND_SOFT } else { Color32::TRANSPARENT })
                        .stroke(Stroke::NONE)
                        .min_size(egui::vec2(ui.available_width(), 38.0));
                    if ui.add(btn).clicked() {
                        self.tab = tab;
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    if ui.add(egui::Button::new("Sign out").min_size(egui::vec2(ui.available_width(), 34.0))).clicked() {
                        self.sign_out();
                    }
                    ui.add_space(6.0);
                    ui.label(RichText::new(format!("Version {}", config::APP_VERSION)).color(FAINT).size(12.0));
                    ui.label(RichText::new(self.profile.name()).color(MUTED));
                    if self.loading > 0 {
                        ui.spinner();
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(SURFACE).inner_margin(Margin::same(24)))
            .show(ctx, |ui| {
                if maintenance {
                    banner(ui, "QuickBiz is down for maintenance. Some things may not work for a short while.", AMBER, AMBER_SOFT);
                }
                if let Some(v) = update {
                    let text = if config::STORE_BUILD {
                        format!("Version {v} is available. Update it from the Microsoft Store.")
                    } else {
                        format!("Version {v} is available on the QuickBiz website.")
                    };
                    banner(ui, &text, BRAND, BRAND_SOFT);
                }
                // personal / business switch
                ui.horizontal(|ui| {
                    for (m, label) in [("business", "Business"), ("personal", "Personal")] {
                        if ui.selectable_label(self.mode == m, label).clicked() && self.mode != m {
                            self.mode = m.into();
                            self.txs.clear();
                            self.reload(ctx);
                        }
                    }
                });
                ui.add_space(10.0);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.tab {
                    Tab::Dashboard => views::dashboard::show(self, ui),
                    Tab::Finance => views::finance::show(self, ctx, ui),
                    Tab::Inventory => views::inventory::show(self, ctx, ui),
                    Tab::Reports => views::reports::show(self, ui),
                });
            });
    }

    fn toast_ui(&self, ctx: &egui::Context) {
        let Some((text, ok, _)) = &self.toast else { return };
        egui::Area::new(egui::Id::new("toast"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(if *ok { GREEN } else { RED })
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(16, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new(text).color(Color32::WHITE));
                    });
            });
    }
}

// ---- shared widgets ------------------------------------------------------

pub fn banner(ui: &mut egui::Ui, text: &str, fg: Color32, bg: Color32) {
    egui::Frame::new().fill(bg).corner_radius(CornerRadius::same(10)).inner_margin(Margin::symmetric(14, 10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(text).color(fg));
    });
    ui.add_space(8.0);
}

pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(WHITE)
        .stroke(Stroke::new(1.0_f32, LINE))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

pub fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).color(Color32::WHITE).font(bold_font(14.5)))
        .fill(BRAND)
        .stroke(Stroke::NONE)
}

pub fn stat(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    card(ui, |ui| {
        ui.label(RichText::new(label).color(MUTED).size(13.0));
        ui.label(RichText::new(value).font(bold_font(24.0)).color(color));
    });
}

pub fn heading(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).font(bold_font(24.0)));
    ui.add_space(8.0);
}

/// Parses "1,250.50" or "1250" into a number.
pub fn parse_amount(s: &str) -> Option<f64> {
    s.trim().replace(',', "").parse::<f64>().ok().filter(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_amounts() {
        assert_eq!(parse_amount("1,250.50"), Some(1250.5));
        assert_eq!(parse_amount(" 40 "), Some(40.0));
        assert_eq!(parse_amount("abc"), None);
        assert_eq!(parse_amount("NaN"), None);
    }
}
