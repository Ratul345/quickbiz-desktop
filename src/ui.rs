//! Shared widgets that give every screen the same look as the QuickBiz mobile app:
//! soft cards, tinted icon boxes, pills, segmented switches, menu rows, toggles and dialogs.

use crate::theme::*;
use eframe::egui::{self, Color32, CornerRadius, Margin, Response, RichText, Sense, Stroke, Ui};

pub const RADIUS: u8 = 16;

/// The white rounded card used everywhere.
pub fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(WHITE)
        .stroke(Stroke::new(1.0_f32, LINE))
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(Margin::same(18))
        .shadow(egui::epaint::Shadow { offset: [0, 2], blur: 10, spread: 0, color: Color32::from_black_alpha(8) })
}

pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    card_frame()
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// A card with a bold title and an optional link-style action on the right.
pub fn titled_card<R>(ui: &mut Ui, title: &str, action: Option<&str>, add: impl FnOnce(&mut Ui) -> R) -> (R, bool) {
    let mut clicked = false;
    let r = card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).font(bold_font(16.5)));
            if let Some(a) = action {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if link(ui, a).clicked() {
                        clicked = true;
                    }
                });
            }
        });
        ui.add_space(8.0);
        add(ui)
    });
    (r, clicked)
}

/// Text in the icon font. Returns an empty label when Segoe MDL2 Assets is missing.
pub fn icon_text(glyph: &str, size: f32, color: Color32) -> RichText {
    if icons_available() {
        RichText::new(glyph).font(icon_font(size)).color(color)
    } else {
        RichText::new("").size(size)
    }
}

pub fn icon(ui: &mut Ui, glyph: &str, size: f32, color: Color32) -> Response {
    ui.add(egui::Label::new(icon_text(glyph, size, color)).selectable(false))
}

/// Draws a glyph centred in `rect`.
pub fn paint_icon(ui: &Ui, rect: egui::Rect, glyph: &str, size: f32, color: Color32) {
    if icons_available() {
        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, glyph, icon_font(size), color);
    }
}

/// A rounded square with a soft tint and a coloured icon, like the mobile menu icons.
pub fn icon_box(ui: &mut Ui, glyph: &str, color: Color32, side: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(side, side), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same((side * 0.3) as u8), tint(color, 0.12));
    paint_icon(ui, rect, glyph, side * 0.46, color);
    resp
}

/// A round version of [`icon_box`].
pub fn icon_circle(ui: &mut Ui, glyph: &str, color: Color32, side: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(side, side), Sense::hover());
    ui.painter().circle_filled(rect.center(), side / 2.0, tint(color, 0.12));
    paint_icon(ui, rect, glyph, side * 0.44, color);
    resp
}

/// A small rounded label, e.g. "BUSINESS", "PRO", "Low stock".
pub fn pill(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) -> Response {
    egui::Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(255))
        .inner_margin(Margin::symmetric(10, 3))
        .show(ui, |ui| ui.label(RichText::new(text).font(bold_font(11.5)).color(fg)))
        .response
}

/// A caption above a group of settings, e.g. "ACCOUNT SETTINGS".
pub fn section_title(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text.to_uppercase()).font(bold_font(11.5)).color(FAINT).extra_letter_spacing(1.2));
    ui.add_space(2.0);
}

/// "‹ Back to Profile" style link used on sub-pages.
pub fn back_link(ui: &mut Ui, text: &str) -> bool {
    let r = ui
        .horizontal(|ui| {
            icon(ui, ic::CHEVRON_LEFT, 12.0, BRAND);
            ui.add(egui::Label::new(RichText::new(text).color(BRAND).font(bold_font(14.0))).sense(Sense::click()))
        })
        .inner;
    let r = r.on_hover_cursor(egui::CursorIcon::PointingHand);
    ui.add_space(6.0);
    r.clicked()
}

pub fn link(ui: &mut Ui, text: &str) -> Response {
    ui.add(egui::Label::new(RichText::new(text).color(BRAND).font(bold_font(13.5))).sense(Sense::click()))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn button(text: &str, fg: Color32, bg: Color32, stroke: Stroke) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).color(fg).font(bold_font(14.5)))
        .fill(bg)
        .stroke(stroke)
        .corner_radius(CornerRadius::same(12))
        .min_size(egui::vec2(0.0, 40.0))
}

pub fn primary_button(text: &str) -> egui::Button<'static> {
    button(text, WHITE, BRAND, Stroke::NONE)
}

pub fn secondary_button(text: &str) -> egui::Button<'static> {
    button(text, INK, WHITE, Stroke::new(1.0_f32, LINE_STRONG))
}

pub fn danger_button(text: &str) -> egui::Button<'static> {
    button(text, WHITE, RED, Stroke::NONE)
}

pub fn soft_danger_button(text: &str) -> egui::Button<'static> {
    button(text, RED, WHITE, Stroke::new(1.0_f32, tint(RED, 0.3)))
}

/// A pill-shaped switch between options. Returns true when the choice changed.
pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) -> bool {
    let mut changed = false;
    egui::Frame::new()
        .fill(SURFACE_MUTED)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for (v, label) in options {
                    let on = *value == *v;
                    let text = RichText::new(*label).font(bold_font(13.5)).color(if on { INK } else { MUTED });
                    let b = egui::Button::new(text)
                        .fill(if on { WHITE } else { Color32::TRANSPARENT })
                        .stroke(if on { Stroke::new(1.0_f32, LINE) } else { Stroke::NONE })
                        .corner_radius(CornerRadius::same(9))
                        .min_size(egui::vec2(0.0, 32.0));
                    if ui.add(b).clicked() && !on {
                        *value = *v;
                        changed = true;
                    }
                }
            });
        });
    changed
}

/// A chip that can be selected (currency, category, message type).
pub fn chip(ui: &mut Ui, label: &str, on: bool) -> Response {
    let text = RichText::new(label).font(bold_font(13.0)).color(if on { BRAND } else { MUTED });
    ui.add(
        egui::Button::new(text)
            .fill(if on { BRAND_FAINT } else { WHITE })
            .stroke(Stroke::new(1.0_f32, if on { BRAND } else { LINE_STRONG }))
            .corner_radius(CornerRadius::same(255))
            .min_size(egui::vec2(0.0, 34.0)),
    )
}

/// A row in a settings list: icon box, title, subtitle, optional badge and a chevron.
pub fn menu_row(ui: &mut Ui, glyph: &str, color: Color32, title: &str, subtitle: Option<&str>, badge: Option<&str>) -> Response {
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 60.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(12), SURFACE);
    }
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(egui::vec2(10.0, 0.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
    icon_box(&mut child, glyph, color, 38.0);
    child.add_space(6.0);
    child.vertical(|ui| {
        ui.add_space(if subtitle.is_some() { 10.0 } else { 20.0 });
        ui.spacing_mut().item_spacing.y = 1.0;
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).font(bold_font(14.5)).color(if color == RED { RED } else { INK }));
            if let Some(b) = badge {
                pill(ui, b, WHITE, BRAND);
            }
        });
        if let Some(s) = subtitle {
            ui.label(RichText::new(s).size(12.5).color(MUTED));
        }
    });
    let chevron = egui::Rect::from_center_size(egui::pos2(rect.right() - 18.0, rect.center().y), egui::vec2(16.0, 16.0));
    paint_icon(ui, chevron, ic::CHEVRON_RIGHT, 11.0, FAINT);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A thin divider line inside a card.
pub fn divider(ui: &mut Ui) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::ZERO, LINE);
}

/// An on/off switch. Returns true when flipped.
pub fn toggle(ui: &mut Ui, on: &mut bool) -> bool {
    let size = egui::vec2(44.0, 24.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let t = ui.ctx().animate_bool_responsive(resp.id, *on);
    let bg = if *on { BRAND } else { LINE_STRONG };
    ui.painter().rect_filled(rect, CornerRadius::same(12), bg);
    let x = egui::lerp((rect.left() + 12.0)..=(rect.right() - 12.0), t);
    ui.painter().circle_filled(egui::pos2(x, rect.center().y), 9.0, WHITE);
    let clicked = resp.clicked();
    if clicked {
        *on = !*on;
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    clicked
}

/// A labelled text field.
pub fn field(ui: &mut Ui, label: &str, value: &mut String, hint: &str) -> Response {
    ui.label(RichText::new(label).font(bold_font(13.0)).color(MUTED));
    ui.add(egui::TextEdit::singleline(value).hint_text(hint).desired_width(f32::INFINITY).margin(Margin::symmetric(10, 8)))
}

pub fn multiline(ui: &mut Ui, label: &str, value: &mut String, hint: &str, rows: usize) -> Response {
    ui.label(RichText::new(label).font(bold_font(13.0)).color(MUTED));
    ui.add(egui::TextEdit::multiline(value).hint_text(hint).desired_rows(rows).desired_width(f32::INFINITY).margin(Margin::symmetric(10, 8)))
}

/// A centred dialog over a dimmed window. Returns the content's result and whether it was closed
/// with the X, Escape or a click outside.
pub fn dialog<R>(ctx: &egui::Context, id: &str, title: &str, width: f32, add: impl FnOnce(&mut Ui) -> R) -> (R, bool) {
    let frame = egui::Frame::new()
        .fill(WHITE)
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin::same(24))
        .shadow(egui::epaint::Shadow { offset: [0, 16], blur: 40, spread: 0, color: Color32::from_black_alpha(50) });
    let mut close = false;
    let resp = egui::Modal::new(egui::Id::new(id)).frame(frame).backdrop_color(Color32::from_black_alpha(110)).show(ctx, |ui| {
        ui.set_width(width);
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).font(bold_font(20.0)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let x = ui.add(egui::Button::new(icon_text(ic::CLOSE, 12.0, MUTED)).fill(SURFACE_MUTED).corner_radius(CornerRadius::same(255)).min_size(egui::vec2(30.0, 30.0)));
                if x.clicked() {
                    close = true;
                }
            });
        });
        ui.add_space(10.0);
        add(ui)
    });
    let should_close = resp.should_close();
    (resp.inner, close || should_close)
}

/// A yes/no question. Returns Some(true) for the confirm button, Some(false) for cancel or close.
pub fn confirm(ctx: &egui::Context, id: &str, title: &str, text: &str, yes: &str, danger: bool) -> Option<bool> {
    let (choice, closed) = dialog(ctx, id, title, 420.0, |ui| {
        ui.label(RichText::new(text).color(MUTED));
        ui.add_space(16.0);
        let mut choice = None;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let b = if danger { danger_button(yes) } else { primary_button(yes) };
            if ui.add(b.min_size(egui::vec2(120.0, 40.0))).clicked() {
                choice = Some(true);
            }
            if ui.add(secondary_button("Cancel").min_size(egui::vec2(100.0, 40.0))).clicked() {
                choice = Some(false);
            }
        });
        choice
    });
    if closed { Some(false) } else { choice }
}

/// An empty-state block: big tinted icon, title, explanation.
pub fn empty_state(ui: &mut Ui, glyph: &str, title: &str, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(12.0);
        icon_circle(ui, glyph, BRAND, 64.0);
        ui.add_space(8.0);
        ui.label(RichText::new(title).font(bold_font(16.0)));
        ui.label(RichText::new(text).color(MUTED));
        ui.add_space(12.0);
    });
}

/// A stat card with a tinted icon, label and big value.
pub fn stat(ui: &mut Ui, glyph: &str, label: &str, value: &str, color: Color32) {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            icon_box(ui, glyph, color, 40.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(RichText::new(label).color(MUTED).size(12.5));
                ui.label(RichText::new(value).font(bold_font(21.0)).color(INK));
            });
        });
    });
}
