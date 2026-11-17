//! App shell: sign-in state, background loading, the sidebar, the header and shared helpers.
//! Network calls run on worker threads and report back through a channel, so the window never freezes.

use crate::api::{Client, Res, Session};
use crate::config;
use crate::model::*;
use crate::store;
use crate::theme::*;
use crate::ui;
use crate::views;
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Sense, Stroke};
use std::collections::HashMap;
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
    Employees(Res<Vec<Employee>>),
    Found(Res<Option<FoundUser>>),
    Invites(Res<Vec<TeamInvite>>),
    Tickets(Res<Vec<Ticket>>),
    Image(String, Res<egui::ColorImage>),
    /// A picture was chosen and uploaded; the text says what it is for ("profile" or "business").
    Uploaded(&'static str, Res<String>),
    /// A write finished. The text is what to tell the user on success.
    Done(Res<()>, &'static str),
    /// The "forgot password" email was requested.
    ResetSent(Res<()>),
    /// Your account deletion was scheduled; sign out.
    DeletionScheduled(Res<()>),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Tab {
    Dashboard,
    Finance,
    Inventory,
    Reports,
    Profile,
    Team,
    Notifications,
    Support,
}

impl Tab {
    pub fn title(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Finance => "Finance",
            Tab::Inventory => "Inventory",
            Tab::Reports => "Reports",
            Tab::Profile => "Profile",
            Tab::Team => "Team",
            Tab::Notifications => "Notifications",
            Tab::Support => "Help & Support",
        }
    }
    /// Which sidebar item is highlighted for this page.
    fn nav(self) -> Tab {
        match self {
            Tab::Notifications | Tab::Support => Tab::Profile,
            t => t,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Stage {
    Booting,
    Login,
    Main,
}

pub enum Img {
    Loading,
    Ready(egui::TextureHandle),
    Failed,
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
    profile_loaded: bool,
    invites_checked: bool,
    pub invite: Option<TeamInvite>,
    pub settings: Settings,
    pub txs: Vec<Transaction>,
    pub products: Vec<Product>,
    pub moves: Vec<StockMovement>,
    pub moves_for: Option<String>,
    pub employees: Vec<Employee>,
    pub tickets: Vec<Ticket>,
    pub images: HashMap<String, Img>,
    pub uploading: Option<&'static str>,
    pub whats_new: bool,
    pub loading: u32,
    pub toast: Option<(String, bool, Instant)>,
    pub last_refresh: Instant,
    pub prefs: store::Prefs,
    pub login: views::login::LoginState,
    pub finance: views::finance::FinanceState,
    pub inventory: views::inventory::InventoryState,
    pub reports: views::reports::ReportsState,
    pub profile_ui: views::profile::ProfileState,
    pub team: views::team::TeamState,
    pub support: views::support::SupportState,
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
            profile_loaded: false,
            invites_checked: false,
            invite: None,
            settings: Settings::default(),
            txs: vec![],
            products: vec![],
            moves: vec![],
            moves_for: None,
            employees: vec![],
            tickets: vec![],
            images: HashMap::new(),
            uploading: None,
            whats_new: false,
            loading: 0,
            toast: None,
            last_refresh: Instant::now(),
            prefs: if cfg!(feature = "demo") { Default::default() } else { store::load_prefs() },
            login: Default::default(),
            finance: Default::default(),
            inventory: Default::default(),
            reports: Default::default(),
            profile_ui: Default::default(),
            team: Default::default(),
            support: Default::default(),
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

    pub fn save_prefs(&self) {
        if !cfg!(feature = "demo") {
            store::save_prefs(&self.prefs);
        }
    }

    pub fn go(&mut self, tab: Tab, ctx: &egui::Context) {
        self.tab = tab;
        match tab {
            Tab::Team => self.load_team(ctx),
            Tab::Support => self.load_tickets(ctx),
            _ => {}
        }
    }

    /// Owner whose data we read and write. Personal mode is always your own; business staff use the owner's.
    pub fn owner(&self) -> String {
        if self.mode == "personal" {
            self.profile.uid.clone()
        } else {
            self.profile.data_owner()
        }
    }

    pub fn business(&self) -> bool {
        self.mode == "business"
    }

    /// Permissions that apply right now. Personal mode is all yours.
    pub fn perms(&self) -> Permissions {
        if self.business() {
            self.profile.perms()
        } else {
            Permissions { can_view_price: true, can_view_stock: true, can_manage_inventory: true, can_manage_transactions: !self.profile.is_employee(), can_view_reports: true }
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

    pub fn reload_profile(&mut self, ctx: &egui::Context) {
        if cfg!(feature = "demo") {
            return;
        }
        let uid = self.profile.uid.clone();
        if uid.is_empty() {
            return;
        }
        self.spawn(ctx, move |c, tx| {
            let _ = tx.send(Msg::Profile(c.profile(&uid)));
        });
    }

    pub fn load_moves(&mut self, ctx: &egui::Context, product_id: String) {
        let owner = self.owner();
        self.moves_for = Some(product_id.clone());
        if cfg!(feature = "demo") {
            return;
        }
        self.spawn(ctx, move |c, tx| {
            let r = c.movements(&owner, &product_id);
            let _ = tx.send(Msg::Moves(product_id, r));
        });
    }

    pub fn load_team(&mut self, ctx: &egui::Context) {
        if cfg!(feature = "demo") || !self.profile.is_owner() {
            return;
        }
        let uid = self.profile.uid.clone();
        self.spawn(ctx, move |c, tx| {
            let _ = tx.send(Msg::Employees(c.employees(&uid)));
        });
    }

    pub fn load_tickets(&mut self, ctx: &egui::Context) {
        if cfg!(feature = "demo") {
            return;
        }
        let uid = self.profile.uid.clone();
        self.spawn(ctx, move |c, tx| {
            let _ = tx.send(Msg::Tickets(c.tickets(&uid)));
        });
    }

    /// Runs a write in the background, then tells the user and refreshes everything.
    pub fn write<F>(&mut self, ctx: &egui::Context, ok_text: &'static str, f: F)
    where
        F: FnOnce(&Client) -> Res<()> + Send + 'static,
    {
        self.spawn(ctx, move |c, tx| {
            let _ = tx.send(Msg::Done(f(c), ok_text));
        });
    }

    /// Personal or business. Switching to business without a business asks you to create one first.
    pub fn set_mode(&mut self, ctx: &egui::Context, m: &str) {
        if self.mode == m {
            return;
        }
        if m == "business" && !self.profile.has_business() {
            self.profile_ui.open_business(&self.profile, true);
            return;
        }
        self.mode = m.into();
        self.txs.clear();
        self.products.clear();
        self.reload(ctx);
    }

    /// The texture for a picture at `url`, downloading it the first time it is asked for.
    pub fn image(&mut self, ctx: &egui::Context, url: &str) -> Option<egui::TextureHandle> {
        match self.images.get(url) {
            Some(Img::Ready(t)) => return Some(t.clone()),
            Some(_) => return None,
            None => {}
        }
        self.images.insert(url.to_string(), Img::Loading);
        let u = url.to_string();
        self.spawn(ctx, move |c, tx| {
            let r = c.download(&u).and_then(|bytes| crate::pictures::thumbnail(&bytes, 256));
            let _ = tx.send(Msg::Image(u, r));
        });
        None
    }

    /// A round picture, or the first letter on a soft circle when there is none.
    pub fn avatar(&mut self, ui: &mut egui::Ui, url: Option<String>, initial: &str, size: f32, color: Color32, sense: Sense) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), sense);
        let tex = url.filter(|u| !u.is_empty()).and_then(|u| self.image(ui.ctx(), &u));
        if let Some(t) = tex {
            egui::Image::new((t.id(), rect.size())).corner_radius(CornerRadius::same((size / 2.0).min(255.0) as u8)).paint_at(ui, rect);
        } else {
            ui.painter().circle_filled(rect.center(), size / 2.0, tint(color, 0.12));
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, initial, bold_font(size * 0.42), color);
        }
        resp
    }

    /// Opens a file picker on a worker thread, shrinks the picture and uploads it.
    pub fn pick_picture(&mut self, ctx: &egui::Context, kind: &'static str) {
        if self.uploading.is_some() || cfg!(feature = "demo") {
            return;
        }
        self.uploading = Some(kind);
        let uid = self.profile.uid.clone();
        let folder = if kind == "profile" { "profiles" } else { "businesses" };
        self.spawn(ctx, move |c, tx| {
            let picked = rfd::FileDialog::new().set_title("Choose a picture").add_filter("Pictures", &["jpg", "jpeg", "png"]).pick_file();
            let r = match picked {
                None => Err(String::new()),
                Some(path) => std::fs::read(&path)
                    .map_err(|e| format!("Could not open the picture: {e}"))
                    .and_then(|bytes| crate::pictures::to_jpeg(&bytes, 512))
                    .and_then(|jpeg| c.upload_image(&uid, folder, &jpeg)),
            };
            let _ = tx.send(Msg::Uploaded(kind, r));
        });
    }

    fn after_sign_in(&mut self, ctx: &egui::Context, s: Session) {
        let uid = s.uid.clone();
        self.profile.uid = uid.clone();
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
        self.profile_loaded = false;
        self.invites_checked = false;
        self.invite = None;
        self.txs.clear();
        self.products.clear();
        self.moves.clear();
        self.employees.clear();
        self.tickets.clear();
        self.profile_ui = Default::default();
        self.team = Default::default();
        self.support = Default::default();
        self.tab = Tab::Dashboard;
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
                Msg::ResetSent(r) => {
                    self.login.busy = false;
                    match r {
                        Ok(()) => {
                            self.login.error = None;
                            self.login.info = Some("If an account exists for that email, a password reset link is on its way.".into());
                        }
                        Err(e) => self.login.error = Some(e),
                    }
                }
                Msg::SignedUp(Err(e)) => {
                    self.login.busy = false;
                    self.login.error = Some(e);
                }
                Msg::Profile(Ok(p)) => {
                    let first = !self.profile_loaded;
                    let before = (self.owner(), self.mode.clone());
                    if first {
                        self.mode = if p.has_business() { "business".into() } else { "personal".into() };
                    } else if self.mode == "business" && !p.has_business() {
                        self.mode = "personal".into(); // left or deleted the business
                    }
                    self.profile = p;
                    self.profile_loaded = true;
                    if first || before != (self.owner(), self.mode.clone()) {
                        self.txs.clear();
                        self.products.clear();
                        self.reload(ctx);
                    }
                    if !self.invites_checked && !self.profile.has_business() {
                        self.invites_checked = true;
                        self.spawn(ctx, |c, tx| {
                            let _ = tx.send(Msg::Invites(c.my_invites()));
                        });
                    }
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
                Msg::Employees(Ok(v)) => self.employees = v,
                Msg::Employees(Err(e)) => self.say(e, false),
                Msg::Found(r) => views::team::found(self, ctx, r),
                Msg::Invites(Ok(v)) => self.invite = v.into_iter().next(),
                Msg::Invites(Err(_)) => {}
                Msg::Tickets(Ok(v)) => self.tickets = v,
                Msg::Tickets(Err(e)) => self.say(e, false),
                Msg::Image(url, r) => {
                    let state = match r {
                        Ok(img) => Img::Ready(ctx.load_texture(url.clone(), img, egui::TextureOptions::LINEAR)),
                        Err(_) => Img::Failed,
                    };
                    self.images.insert(url, state);
                }
                Msg::Uploaded(kind, r) => {
                    self.uploading = None;
                    match r {
                        Err(e) if e.is_empty() => {} // picker cancelled
                        Err(e) => self.say(e, false),
                        Ok(url) => views::profile::uploaded(self, ctx, kind, url),
                    }
                }
                Msg::Done(Ok(()), ok_text) => {
                    self.say(ok_text, true);
                    self.reload(ctx);
                    self.reload_profile(ctx);
                    if let Some(id) = self.moves_for.clone() {
                        self.load_moves(ctx, id);
                    }
                    if self.tab == Tab::Team {
                        self.load_team(ctx);
                    }
                    if self.tab == Tab::Support {
                        self.load_tickets(ctx);
                    }
                }
                Msg::Done(Err(e), _) => {
                    self.say(e, false);
                    self.reload_profile(ctx); // undo anything shown early
                }
                Msg::DeletionScheduled(Ok(())) => {
                    self.sign_out();
                    self.login.info = Some("Your account will be deleted in 30 days. Sign in before then if you change your mind.".into());
                }
                Msg::DeletionScheduled(Err(e)) => self.say(e, false),
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
            Stage::Main => {
                self.main_ui(ctx);
                views::profile::dialogs(self, ctx);
                views::team::dialogs(self, ctx);
                self.invite_ui(ctx);
                views::whatsnew::show(self, ctx);
            }
        }
        self.toast_ui(ctx);
    }
}

const NAV: [(Tab, &str, &str); 5] = [
    (Tab::Dashboard, ic::HOME, "Dashboard"),
    (Tab::Finance, ic::CARD, "Finance"),
    (Tab::Inventory, ic::BOX, "Inventory"),
    (Tab::Reports, ic::CHART, "Reports"),
    (Tab::Profile, ic::PERSON, "Profile"),
];

impl App {
    fn sidebar(&mut self, ctx: &egui::Context) {
        let target = if self.prefs.sidebar_collapsed { 76.0 } else { 240.0 };
        let width = ctx.animate_value_with_time(egui::Id::new("sidebar-width"), target, 0.16);
        let wide = width > 170.0;
        egui::SidePanel::left("nav")
            .exact_width(width)
            .resizable(false)
            .frame(egui::Frame::new().fill(WHITE).inner_margin(Margin::symmetric(14, 16)).stroke(Stroke::new(1.0_f32, LINE)))
            .show(ctx, |ui| {
                // logo and the fold button
                ui.horizontal(|ui| {
                    if wide {
                        if let Some(tex) = crate::logo::texture(ctx) {
                            ui.image((tex.id(), egui::vec2(34.0, 34.0)));
                        }
                        ui.label(RichText::new("QuickBiz").font(bold_font(20.0)).color(BRAND));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if fold_button(ui, ic::CHEVRON_LEFT).on_hover_text("Collapse sidebar").clicked() {
                                self.prefs.sidebar_collapsed = true;
                                self.save_prefs();
                            }
                        });
                    } else {
                        ui.add_space((ui.available_width() - 40.0).max(0.0) / 2.0);
                        if fold_button(ui, ic::MENU).on_hover_text("Expand sidebar").clicked() {
                            self.prefs.sidebar_collapsed = false;
                            self.save_prefs();
                        }
                    }
                });
                ui.add_space(22.0);
                if wide {
                    ui.label(RichText::new("MENU").font(bold_font(11.0)).color(FAINT).extra_letter_spacing(1.2));
                    ui.add_space(4.0);
                }
                for (tab, glyph, label) in NAV {
                    let on = self.tab.nav() == tab;
                    let r = nav_item(ui, glyph, label, on, wide);
                    let r = if wide { r } else { r.on_hover_text(label) };
                    if r.clicked() {
                        self.go(tab, ctx);
                    }
                }
                if self.profile.is_owner() && self.business() {
                    let r = nav_item(ui, ic::PEOPLE, "Team", self.tab == Tab::Team, wide);
                    let r = if wide { r } else { r.on_hover_text("Team") };
                    if r.clicked() {
                        self.go(Tab::Team, ctx);
                    }
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    if wide {
                        ui.label(RichText::new(format!("Version {}", config::APP_VERSION)).color(FAINT).size(11.5));
                        ui.add_space(6.0);
                    }
                    self.user_card(ui, wide);
                    if self.loading > 0 {
                        ui.add_space(6.0);
                        ui.spinner();
                    }
                });
            });
    }

    fn user_card(&mut self, ui: &mut egui::Ui, wide: bool) {
        let url = self.profile.photo_url.clone();
        let initial = self.profile.initial();
        let height = if wide { 64.0 } else { 52.0 };
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::click());
        if resp.hovered() || self.tab == Tab::Profile {
            ui.painter().rect_filled(rect, CornerRadius::same(14), SURFACE);
        }
        ui.painter().rect_stroke(rect, CornerRadius::same(14), Stroke::new(1.0_f32, LINE), egui::StrokeKind::Inside);
        let inner = if wide { rect.shrink2(egui::vec2(10.0, 0.0)) } else { rect };
        let layout = if wide { egui::Layout::left_to_right(egui::Align::Center) } else { egui::Layout::centered_and_justified(egui::Direction::LeftToRight) };
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(layout));
        if wide {
            self.avatar(&mut child, url, &initial, 38.0, BRAND, Sense::hover());
            child.vertical(|ui| {
                ui.add_space(10.0);
                ui.spacing_mut().item_spacing.y = 3.0;
                let name: String = self.profile.name().chars().take(18).collect();
                ui.label(RichText::new(name).font(bold_font(14.0)));
                let (label, fg, bg) = if self.business() { ("BUSINESS", BRAND, BRAND_SOFT) } else { ("PERSONAL", GREEN, GREEN_SOFT) };
                ui.horizontal(|ui| {
                    ui::pill(ui, label, fg, bg);
                });
            });
        } else {
            child.horizontal(|ui| {
                ui.add_space((ui.available_width() - 38.0).max(0.0) / 2.0);
                self.avatar(ui, url, &initial, 38.0, BRAND, Sense::hover());
            });
        }
        let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
        let resp = if wide { resp } else { resp.on_hover_text(self.profile.name()) };
        if resp.clicked() {
            self.go(Tab::Profile, ui.ctx());
        }
    }

    fn main_ui(&mut self, ctx: &egui::Context) {
        self.sidebar(ctx);
        let maintenance = self.settings.maintenance_mode.unwrap_or(false);
        let update = self
            .settings
            .windows_version
            .as_deref()
            .filter(|v| is_newer_version(v, config::APP_VERSION))
            .map(|v| v.to_string());

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(SURFACE).inner_margin(Margin { left: 28, right: 28, top: 20, bottom: 0 }))
            .show(ctx, |ui| {
                if maintenance {
                    banner(ui, ic::WARNING, "QuickBiz is down for maintenance. Some things may not work for a short while.", AMBER, AMBER_SOFT);
                }
                if let Some(v) = update {
                    let text = if config::STORE_BUILD {
                        format!("Version {v} is available. Update it from the Microsoft Store.")
                    } else {
                        format!("Version {v} is available on the QuickBiz website.")
                    };
                    if banner(ui, ic::REFRESH, &text, BRAND, BRAND_SOFT) {
                        open_url(if config::STORE_BUILD { config::STORE_URL } else { config::WEB_URL });
                    }
                }
                self.header(ui, ctx);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    // keep cards clear of the scroll bar
                    ui.set_max_width(ui.available_width() - 14.0);
                    ui.add_space(4.0);
                    match self.tab {
                        Tab::Dashboard => views::dashboard::show(self, ctx, ui),
                        Tab::Finance => views::finance::show(self, ctx, ui),
                        Tab::Inventory => views::inventory::show(self, ctx, ui),
                        Tab::Reports => views::reports::show(self, ui),
                        Tab::Profile => views::profile::show(self, ctx, ui),
                        Tab::Team => views::team::show(self, ctx, ui),
                        Tab::Notifications => views::notifications::show(self, ctx, ui),
                        Tab::Support => views::support::show(self, ctx, ui),
                    }
                    ui.add_space(28.0);
                });
            });
    }

    /// Page title on the left; personal/business switch on the right.
    fn header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                if self.tab == Tab::Dashboard {
                    let hour = chrono::Timelike::hour(&chrono::Local::now());
                    let greet = if hour < 12 { "Good morning" } else if hour < 17 { "Good afternoon" } else { "Good evening" };
                    let first = self.profile.first_name();
                    let title = if first.is_empty() { greet.to_string() } else { format!("{greet}, {first}") };
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(title).font(bold_font(26.0)));
                        if self.profile.is_verified.unwrap_or(false) {
                            ui::icon(ui, ic::CHECK_CIRCLE, 16.0, BRAND);
                        }
                        if self.profile.is_pro.unwrap_or(false) {
                            ui::pill(ui, "PRO", GOLD, GOLD_SOFT);
                        }
                    });
                    let ctx_line = if self.profile.is_employee() && self.business() {
                        format!("Working at {}", self.profile.business_name.clone().unwrap_or_else(|| "the business".into()))
                    } else if self.business() {
                        format!("Managing {}", self.profile.business_name.clone().unwrap_or_else(|| "your business".into()))
                    } else {
                        chrono::Local::now().format("%A, %d %B").to_string()
                    };
                    ui.label(RichText::new(ctx_line).color(MUTED));
                } else {
                    ui.label(RichText::new(self.tab.title()).font(bold_font(26.0)));
                    let sub = match self.tab {
                        Tab::Finance => "Record and review income and expenses",
                        Tab::Inventory => "Products, stock levels and history",
                        Tab::Reports => "How your money moved over time",
                        Tab::Profile => "Your account, business and settings",
                        Tab::Team => "Invite employees and choose what they can do",
                        Tab::Notifications => "Choose which alerts your phone shows",
                        Tab::Support => "Questions, problems and ideas, straight to our team",
                        Tab::Dashboard => "",
                    };
                    ui.label(RichText::new(sub).color(MUTED));
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut m = if self.business() { 0 } else { 1 };
                if ui::segmented(ui, &mut m, &[(1, "Personal"), (0, "Business")]) {
                    self.set_mode(ctx, if m == 0 { "business" } else { "personal" });
                }
            });
        });
        ui.add_space(14.0);
    }

    fn invite_ui(&mut self, ctx: &egui::Context) {
        let Some(inv) = self.invite.clone() else { return };
        let who = inv.owner_name.clone().unwrap_or_else(|| "A business owner".into());
        let business = inv.business_name.clone().map(|b| format!(" ({b})")).unwrap_or_default();
        let text = format!(
            "{who}{business} invited you to join their business as an employee. If you accept, they can see your profile details. You can leave at any time from your Profile."
        );
        let mut answer: Option<bool> = None;
        let (_, closed) = ui::dialog(ctx, "team-invite", "Team invitation", 440.0, |ui| {
            ui.horizontal(|ui| {
                ui::icon_circle(ui, ic::PEOPLE, BRAND, 48.0);
                ui.add(egui::Label::new(RichText::new(text).color(MUTED)).wrap());
            });
            ui.add_space(16.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(ui::primary_button("Accept").min_size(egui::vec2(120.0, 40.0))).clicked() {
                    answer = Some(true);
                }
                if ui.add(ui::secondary_button("Decline").min_size(egui::vec2(110.0, 40.0))).clicked() {
                    answer = Some(false);
                }
            });
        });
        if closed && answer.is_none() {
            self.invite = None; // asked again next time you sign in
            return;
        }
        if let Some(accept) = answer {
            self.invite = None;
            let id = inv.id.clone();
            self.write(ctx, if accept { "You joined the team." } else { "Invitation declined." }, move |c| c.answer_invite(&id, accept));
        }
    }

    fn toast_ui(&self, ctx: &egui::Context) {
        let Some((text, ok, _)) = &self.toast else { return };
        egui::Area::new(egui::Id::new("toast"))
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -24.0))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(INK)
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::symmetric(16, 11))
                    .shadow(egui::epaint::Shadow { offset: [0, 8], blur: 24, spread: 0, color: Color32::from_black_alpha(40) })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui::icon(ui, if *ok { ic::CHECK_CIRCLE } else { ic::WARNING }, 15.0, if *ok { Color32::from_rgb(0x4A, 0xDE, 0x80) } else { Color32::from_rgb(0xFC, 0xA5, 0xA5) });
                            ui.label(RichText::new(text).color(Color32::WHITE));
                        });
                    });
            });
    }
}

fn fold_button(ui: &mut egui::Ui, glyph: &str) -> egui::Response {
    ui.add(egui::Button::new(ui::icon_text(glyph, 13.0, MUTED)).fill(SURFACE_MUTED).stroke(Stroke::NONE).corner_radius(CornerRadius::same(10)).min_size(egui::vec2(34.0, 34.0)))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn nav_item(ui: &mut egui::Ui, glyph: &str, label: &str, on: bool, wide: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), Sense::click());
    let bg = if on { BRAND_SOFT } else if resp.hovered() { SURFACE } else { Color32::TRANSPARENT };
    ui.painter().rect_filled(rect, CornerRadius::same(12), bg);
    let color = if on { BRAND } else { MUTED };
    if wide {
        let icon_rect = egui::Rect::from_center_size(egui::pos2(rect.left() + 24.0, rect.center().y), egui::vec2(20.0, 20.0));
        ui::paint_icon(ui, icon_rect, glyph, 17.0, color);
        let font = if on { bold_font(14.5) } else { text_font(14.5) };
        ui.painter().text(egui::pos2(rect.left() + 46.0, rect.center().y), egui::Align2::LEFT_CENTER, label, font, if on { BRAND } else { INK });
    } else {
        ui::paint_icon(ui, rect, glyph, 18.0, color);
    }
    ui.add_space(2.0);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A coloured notice across the top. Returns true when clicked.
pub fn banner(ui: &mut egui::Ui, glyph: &str, text: &str, fg: Color32, bg: Color32) -> bool {
    let r = egui::Frame::new().fill(bg).corner_radius(CornerRadius::same(12)).inner_margin(Margin::symmetric(14, 10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui::icon(ui, glyph, 14.0, fg);
            ui.label(RichText::new(text).color(fg).font(bold_font(13.5)));
        });
    });
    ui.add_space(10.0);
    r.response.interact(Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

/// Opens a web page or Store link in the default app.
pub fn open_url(url: &str) {
    let _ = std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", url]).spawn();
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
