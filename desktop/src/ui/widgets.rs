//! Custom-painted widgets that reproduce the original app's look.

use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, CursorIcon, Frame, Id, Margin, Rect, Response,
    Sense, Shadow, Stroke, StrokeKind, TextEdit, TextureHandle, TextureOptions, Ui, Vec2, pos2,
    text::LayoutJob, vec2,
};
use eframe::egui::{Popup, PopupCloseBehavior};

use super::theme::*;
use crate::dates;
use crate::fonts::Weight;

/* ------------------------------------------------------------------------- */
/* Icons                                                                     */
/* ------------------------------------------------------------------------- */

macro_rules! icon_list {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_bytes!(concat!("../../assets/icons/", $name, ".svg")) as &[u8])),*]
    };
}

static ICON_SVGS: &[(&str, &[u8])] = icon_list![
    "arrow-square-in", "bank", "calendar-blank", "caret-down", "caret-left", "caret-right",
    "chart-bar", "check", "check-circle", "copy", "corners-out", "credit-card", "currency-dollar",
    "database", "download-simple", "envelope-simple", "eye", "eye-slash", "file-code", "file-csv",
    "file-pdf", "floppy-disk", "folder", "gear", "image", "info", "magnifying-glass", "map-pin",
    "minus", "note", "pencil-simple", "phone", "plus", "receipt", "shield-check", "trash",
    "upload-simple", "user", "user-plus", "users", "warning", "warning-circle", "x", "x-circle",
];

/// SVG icons rendered on demand at the exact device-pixel size they are shown
/// at (cached per size), so small icons stay crisp instead of being a blurry
/// downscale of one large bitmap.
pub struct Icons {
    svgs: HashMap<&'static str, &'static [u8]>,
    cache: std::cell::RefCell<HashMap<(&'static str, u32), TextureHandle>>,
    ctx: egui::Context,
}

impl Icons {
    pub fn load(ctx: &egui::Context) -> Self {
        Self {
            svgs: ICON_SVGS.iter().map(|(n, s)| (*n, *s)).collect(),
            cache: Default::default(),
            ctx: ctx.clone(),
        }
    }

    fn texture(&self, name: &str, px: u32) -> Option<egui::TextureId> {
        let (&key, &svg) = self.svgs.get_key_value(name)?;
        let mut cache = self.cache.borrow_mut();
        if let Some(t) = cache.get(&(key, px)) {
            return Some(t.id());
        }
        let (rgba, size) = crate::render::svg_to_rgba(svg, px, Some([255, 255, 255]))?;
        let img = egui::ColorImage::from_rgba_premultiplied(size, &rgba);
        let tex = self
            .ctx
            .load_texture(format!("icon-{key}-{px}"), img, TextureOptions::LINEAR);
        let id = tex.id();
        cache.insert((key, px), tex);
        Some(id)
    }

    pub fn paint(&self, painter: &egui::Painter, name: &str, rect: Rect, color: Color32) {
        let ppp = painter.ctx().pixels_per_point();
        let px = (rect.width().max(rect.height()) * ppp).round().max(1.0) as u32;
        let Some(id) = self.texture(name, px) else { return };
        // Snap to whole device pixels so the bitmap maps 1:1 onto the screen.
        let size = px as f32 / ppp;
        let min = rect.center() - Vec2::splat(size / 2.0);
        let min = pos2((min.x * ppp).round() / ppp, (min.y * ppp).round() / ppp);
        painter.image(
            id,
            Rect::from_min_size(min, Vec2::splat(size)),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            color,
        );
    }

    /// Allocates space and draws an icon inline.
    pub fn show(&self, ui: &mut Ui, name: &str, size: f32, color: Color32) -> Response {
        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        self.paint(ui.painter(), name, rect, color);
        resp
    }
}

/* ------------------------------------------------------------------------- */
/* Text helpers                                                              */
/* ------------------------------------------------------------------------- */

pub fn galley(ui: &Ui, text: &str, size: f32, w: Weight, color: Color32) -> std::sync::Arc<egui::Galley> {
    ui.painter().layout_no_wrap(text.to_string(), font(size, w), color)
}

pub fn text(ui: &mut Ui, text: &str, size: f32, w: Weight, color: Color32) -> Response {
    ui.label(egui::RichText::new(text).font(font(size, w)).color(color))
}

/// Text that wraps to the available width.
pub fn wrapped(ui: &mut Ui, s: &str, size: f32, w: Weight, color: Color32) -> Response {
    // Labels inherit justification from the parent layout (e.g. columns), so
    // place them in a plain left-aligned layout.
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.add(egui::Label::new(egui::RichText::new(s).font(font(size, w)).color(color)).wrap())
    })
    .inner
}

/// Like `ui.horizontal`, but the row is only as tall as its content
/// (egui's default minimum is the 36px widget height).
pub fn tight_row<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let saved = ui.spacing().interact_size.y;
    ui.spacing_mut().interact_size.y = 0.0;
    let r = ui.horizontal(|ui| {
        ui.spacing_mut().interact_size.y = saved;
        add(ui)
    });
    ui.spacing_mut().interact_size.y = saved;
    r.inner
}

/// Uppercase section heading with letter spacing (e.g. "INVOICE DETAILS").
pub fn section_heading(ui: &mut Ui, icons: Option<(&Icons, &str)>, s: &str) {
    tight_row(ui, |ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if let Some((icons, name)) = icons {
            icons.show(ui, name, 15.0, TEXT_DIM);
        }
        let mut job = LayoutJob::default();
        job.append(
            &s.to_uppercase(),
            0.0,
            egui::TextFormat {
                font_id: font(13.0, Weight::SemiBold),
                color: TEXT_DIM,
                extra_letter_spacing: 0.7,
                ..Default::default()
            },
        );
        ui.label(job);
    });
}

/// Small grey label above an input, with an optional required star.
pub fn field_label(ui: &mut Ui, label: &str, required: bool) {
    tight_row(ui, |ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        text(ui, label, 12.0, Weight::Regular, TEXT_MUTED);
        if required {
            text(ui, "*", 12.0, Weight::Regular, BRAND);
        }
    });
}

/* ------------------------------------------------------------------------- */
/* Buttons                                                                   */
/* ------------------------------------------------------------------------- */

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Primary,
    Secondary,
    Tertiary,
    Danger,
}

pub struct Btn<'a> {
    pub kind: Kind,
    pub icon: Option<&'a str>,
    pub label: &'a str,
    pub height: f32,
    pub width: Option<f32>,
    pub font_size: f32,
    pub enabled: bool,
}

impl<'a> Btn<'a> {
    pub fn new(kind: Kind, icon: Option<&'a str>, label: &'a str) -> Self {
        Self {
            kind,
            icon,
            label,
            height: 38.0,
            width: None,
            font_size: 13.0,
            enabled: true,
        }
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    pub fn full(mut self, ui: &Ui) -> Self {
        self.width = Some(ui.available_width());
        self
    }
    pub fn enabled(mut self, e: bool) -> Self {
        self.enabled = e;
        self
    }

    pub fn show(self, ui: &mut Ui, icons: &Icons) -> Response {
        let weight = if self.kind == Kind::Primary { Weight::SemiBold } else { Weight::Medium };
        let g = galley(ui, self.label, self.font_size, weight, TEXT_WHITE);
        let icon_size = self.font_size + 2.0;
        let gap = if self.label.is_empty() { 0.0 } else { 7.0 };
        let content_w = g.size().x + self.icon.map(|_| icon_size + gap).unwrap_or(0.0);
        let w = self.width.unwrap_or(content_w + 32.0);
        let sense = if self.enabled { Sense::click() } else { Sense::hover() };
        let (rect, resp) = ui.allocate_exact_size(vec2(w, self.height), sense);
        let hovered = self.enabled && resp.hovered();
        let pressed = self.enabled && resp.is_pointer_button_down_on();

        let (bg, border, fg) = match self.kind {
            Kind::Primary => {
                let bg = if pressed {
                    BRAND_ACTIVE
                } else if hovered {
                    BRAND_HOVER
                } else {
                    BRAND
                };
                (bg, bg, TEXT_WHITE)
            }
            Kind::Secondary => {
                if pressed {
                    (N840, BORDER_LIGHT, TEXT_WHITE)
                } else if hovered {
                    (N780, BORDER_LIGHT, TEXT_WHITE)
                } else {
                    (BG_SURFACE_HOVER, N720, TEXT_PRIMARY)
                }
            }
            Kind::Tertiary => {
                if hovered {
                    (BG_SURFACE_HOVER, BORDER_DEFAULT, TEXT_WHITE)
                } else {
                    (Color32::TRANSPARENT, BORDER_MUTED, TEXT_SECONDARY)
                }
            }
            Kind::Danger => {
                if hovered {
                    (alpha(DANGER, 0.16), alpha(DANGER, 0.6), DANGER_TEXT)
                } else {
                    (alpha(DANGER, 0.08), alpha(DANGER, 0.35), DANGER_TEXT)
                }
            }
        };
        let fg = if self.enabled { fg } else { alpha(fg, 0.4) };
        let painter = ui.painter();
        let rect = if pressed { rect.shrink(0.5) } else { rect };
        if self.kind == Kind::Primary && hovered {
            painter.add(
                Shadow {
                    offset: [0, 3],
                    blur: 10,
                    spread: 0,
                    color: alpha(BRAND, 0.35),
                }
                .as_shape(rect, CornerRadius::same(6)),
            );
        }
        painter.rect(rect, 6, bg, Stroke::new(1.0, border), StrokeKind::Inside);

        let mut x = rect.center().x - content_w / 2.0;
        if let Some(icon) = self.icon {
            let r = Rect::from_min_size(pos2(x, rect.center().y - icon_size / 2.0), Vec2::splat(icon_size));
            icons.paint(painter, icon, r, fg);
            x += icon_size + gap;
        }
        let g = galley(ui, self.label, self.font_size, weight, fg);
        ui.painter()
            .galley(pos2(x, rect.center().y - g.size().y / 2.0), g, fg);
        if self.enabled {
            resp.on_hover_cursor(CursorIcon::PointingHand)
        } else {
            resp
        }
    }
}

/// Square icon-only button (zoom controls, remove item, …).
pub fn icon_button(
    ui: &mut Ui,
    icons: &Icons,
    icon: &str,
    size: Vec2,
    icon_size: f32,
    color: Color32,
    hover_bg: Color32,
    border: Option<Color32>,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let bg = if resp.hovered() { hover_bg } else { Color32::TRANSPARENT };
    painter.rect(
        rect,
        6,
        bg,
        border.map(|c| Stroke::new(1.0, c)).unwrap_or(Stroke::NONE),
        StrokeKind::Inside,
    );
    let ir = Rect::from_center_size(rect.center(), Vec2::splat(icon_size));
    icons.paint(painter, icon, ir, if resp.hovered() { TEXT_WHITE } else { color });
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/* ------------------------------------------------------------------------- */
/* Inputs                                                                    */
/* ------------------------------------------------------------------------- */

fn input_frame_fix(ui: &Ui, resp: &Response, invalid: bool) {
    if invalid {
        ui.painter().rect_stroke(resp.rect, 8, Stroke::new(1.0, BRAND), StrokeKind::Inside);
        ui.painter().rect_stroke(
            resp.rect.expand(2.0),
            10,
            Stroke::new(3.0, alpha(BRAND, 0.2)),
            StrokeKind::Inside,
        );
    } else if resp.has_focus() {
        ui.painter().rect_stroke(
            resp.rect.expand(3.0),
            11,
            Stroke::new(3.0, alpha(BRAND, 0.15)),
            StrokeKind::Inside,
        );
    }
}

/// Height shared by text inputs, selects and the date field.
pub const INPUT_HEIGHT: f32 = 42.0;

pub fn input(ui: &mut Ui, value: &mut String, hint: &str, invalid: bool) -> Response {
    let resp = ui.add(
        TextEdit::singleline(value)
            .hint_text(egui::RichText::new(hint).color(TEXT_SUBTLE))
            .margin(Margin::symmetric(14, 0))
            .min_size(vec2(0.0, INPUT_HEIGHT))
            .align(Align2::LEFT_CENTER)
            .desired_width(f32::INFINITY)
            .font(font(13.5, Weight::Regular)),
    );
    input_frame_fix(ui, &resp, invalid);
    resp
}

pub fn password_input(ui: &mut Ui, value: &mut String, hint: &str, show: bool) -> Response {
    let resp = ui.add(
        TextEdit::singleline(value)
            .hint_text(egui::RichText::new(hint).color(TEXT_SUBTLE))
            .password(!show)
            .margin(Margin {
                left: 14,
                right: 44,
                top: 0,
                bottom: 0,
            })
            .min_size(vec2(0.0, INPUT_HEIGHT))
            .align(Align2::LEFT_CENTER)
            .desired_width(f32::INFINITY)
            .font(font(13.5, Weight::Regular)),
    );
    input_frame_fix(ui, &resp, false);
    resp
}

pub fn textarea(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    rows: usize,
    size: f32,
    color: Color32,
    invalid: bool,
) -> Response {
    // Comfortable line spacing (~1.55x) for multi-line text.
    let line_height = (size * 1.55).round();
    let mut layouter = |ui: &Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
        let mut job = LayoutJob::single_section(
            buf.as_str().to_owned(),
            egui::TextFormat {
                font_id: font(size, Weight::Regular),
                color,
                line_height: Some(line_height),
                ..Default::default()
            },
        );
        job.wrap.max_width = wrap_width;
        ui.painter().layout_job(job)
    };
    let resp = ui.add(
        TextEdit::multiline(value)
            .hint_text(egui::RichText::new(hint).color(TEXT_SUBTLE).font(font(size, Weight::Regular)))
            .margin(Margin::symmetric(14, 11))
            .desired_rows(rows)
            .min_size(vec2(0.0, rows as f32 * line_height + 22.0))
            .desired_width(f32::INFINITY)
            .layouter(&mut layouter),
    );
    input_frame_fix(ui, &resp, invalid);
    resp
}

/// Labelled single-line input in a vertical group.
pub fn labeled_input(ui: &mut Ui, label: &str, required: bool, value: &mut String, hint: &str, invalid: bool) -> Response {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        field_label(ui, label, required);
        input(ui, value, hint, invalid)
    })
    .inner
}

/// Runs `f` for two side-by-side columns with a 16 px gap.
pub fn two_columns(ui: &mut Ui, f: impl FnOnce(&mut Ui, &mut Ui)) {
    let saved = ui.spacing().item_spacing.x;
    ui.spacing_mut().item_spacing.x = 16.0;
    ui.columns(2, |cols| {
        let (a, b) = cols.split_at_mut(1);
        a[0].spacing_mut().item_spacing.x = saved;
        b[0].spacing_mut().item_spacing.x = saved;
        f(&mut a[0], &mut b[0]);
    });
    ui.spacing_mut().item_spacing.x = saved;
}

/* ------------------------------------------------------------------------- */
/* Select                                                                    */
/* ------------------------------------------------------------------------- */

/// Space between a field and the dropdown / calendar it opens.
const POPUP_GAP: f32 = 8.0;

fn popup_frame() -> Frame {
    Frame::new()
        .fill(BG_SURFACE)
        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(8)
        .inner_margin(Margin::same(5))
        .shadow(Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color: Color32::from_black_alpha(150),
        })
}

/// Option row inside a dropdown/menu. Returns the response.
fn option_row(ui: &mut Ui, label: &str, selected: bool, color: Color32, height: f32) -> Response {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(w, height), Sense::click());
    let bg = if selected {
        alpha(BRAND, 0.14)
    } else if resp.hovered() {
        BG_SURFACE_HOVER
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 6, bg);
    let fg = if selected { BRAND } else if resp.hovered() { TEXT_WHITE } else { color };
    let g = galley(ui, label, 13.0, if selected { Weight::Medium } else { Weight::Regular }, fg);
    ui.painter()
        .galley(pos2(rect.left() + 12.0, rect.center().y - g.size().y / 2.0), g, fg);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// Styled dropdown. Returns `Some(index)` when an option is picked.
pub fn select(
    ui: &mut Ui,
    icons: &Icons,
    id_salt: impl egui::AsIdSalt,
    options: &[String],
    selected: Option<usize>,
    placeholder: &str,
    width: f32,
    height: f32,
    text_color: Option<Color32>,
    invalid: bool,
) -> Option<usize> {
    let id = ui.make_persistent_id(id_salt);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
    let popup_id = id.with("popup");
    // Design-review mode (JHA_REVIEW_OPEN=select): open the first dropdown once.
    if std::env::var("JHA_REVIEW_OPEN").as_deref() == Ok("select") {
        let flag = egui::Id::unique("review-open-select");
        if !ui.ctx().data(|d| d.get_temp::<bool>(flag).unwrap_or(false)) {
            ui.ctx().data_mut(|d| d.insert_temp(flag, true));
            Popup::open_id(ui.ctx(), popup_id);
        }
    }
    let open = Popup::is_id_open(ui.ctx(), popup_id);
    let painter = ui.painter();
    let (bg, border) = if open || invalid {
        (BG_SURFACE, BRAND)
    } else if resp.hovered() {
        (BG_INPUT_HOVER, BORDER_DEFAULT)
    } else {
        (BG_SURFACE, BORDER_SUBTLE)
    };
    if open || invalid {
        painter.rect_stroke(rect.expand(2.0), 10, Stroke::new(3.0, alpha(BRAND, 0.18)), StrokeKind::Inside);
    }
    painter.rect(rect, 8, bg, Stroke::new(1.0, border), StrokeKind::Inside);
    let (label, color) = match selected.and_then(|i| options.get(i)) {
        Some(l) => (l.as_str(), text_color.unwrap_or(TEXT_PRIMARY)),
        None => (placeholder, TEXT_DIM),
    };
    let compact = height < 36.0;
    let fs = if compact { 12.5 } else { 13.5 };
    let pad = if compact { 10.0 } else { 14.0 };
    let caret = 14.0;
    let max_text = rect.width() - pad * 2.0 - caret - 8.0;
    let mut job = LayoutJob::simple_singleline(label.to_string(), font(fs, if compact { Weight::Medium } else { Weight::Regular }), color);
    job.wrap.max_width = max_text;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    let g = ui.painter().layout_job(job);
    ui.painter()
        .galley(pos2(rect.left() + pad, rect.center().y - g.size().y / 2.0), g, color);
    let cr = Rect::from_center_size(pos2(rect.right() - pad - caret / 2.0, rect.center().y), Vec2::splat(caret));
    icons.paint(ui.painter(), "caret-down", cr, if open { BRAND } else { TEXT_DIM });

    let mut picked = None;
    Popup::from_toggle_button_response(&resp)
        .id(popup_id)
        .width(width.max(160.0))
        .gap(POPUP_GAP)
        .align(egui::RectAlign::BOTTOM_START)
        .align_alternatives(&[egui::RectAlign::TOP_START])
        .frame(popup_frame())
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .show(|ui| {
            egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for (i, opt) in options.iter().enumerate() {
                    if option_row(ui, opt, Some(i) == selected, TEXT_SECONDARY, if compact { 30.0 } else { 34.0 }).clicked() {
                        picked = Some(i);
                    }
                }
            });
        });
    picked
}

/* ------------------------------------------------------------------------- */
/* Switch & checkbox                                                         */
/* ------------------------------------------------------------------------- */

pub fn switch(ui: &mut Ui, on: &mut bool) -> Response {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(36.0, 20.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let t = ui.ctx().animate_bool(resp.id, *on);
    let bg = egui::lerp(egui::Rgba::from(N720)..=egui::Rgba::from(BRAND), t);
    ui.painter().rect_filled(rect, 10, Color32::from(bg));
    let x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), t);
    ui.painter().circle_filled(pos2(x, rect.center().y), 7.0, TEXT_WHITE);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// Rounded checkbox. `state`: 0 = off, 1 = on, 2 = indeterminate.
pub fn checkbox(ui: &mut Ui, icons: &Icons, state: u8) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
    let painter = ui.painter();
    if state == 0 {
        let border = if resp.hovered() { BRAND } else { BORDER_DEFAULT };
        painter.rect(rect, 5, BG_SURFACE, Stroke::new(1.5, border), StrokeKind::Inside);
    } else {
        painter.rect_filled(rect, 5, BRAND);
        if state == 1 {
            icons.paint(painter, "check", rect.shrink(3.0), TEXT_WHITE);
        } else {
            painter.line_segment(
                [pos2(rect.left() + 4.5, rect.center().y), pos2(rect.right() - 4.5, rect.center().y)],
                Stroke::new(2.0, TEXT_WHITE),
            );
        }
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// Three vertical dots trigger for row menus.
pub fn dots_button(ui: &mut Ui, open: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::click());
    let painter = ui.painter();
    if resp.hovered() || open {
        painter.rect(rect, 7, BG_SURFACE_HOVER, Stroke::new(1.0, BORDER_MUTED), StrokeKind::Inside);
    }
    let c = if resp.hovered() || open { TEXT_WHITE } else { TEXT_DIM };
    for dy in [-5.0, 0.0, 5.0] {
        painter.circle_filled(rect.center() + vec2(0.0, dy), 1.6, c);
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// Shows a dropdown menu for `trigger`; `items` are (icon, label, danger, separator_before).
/// Returns the index of the clicked item.
pub fn row_menu(
    _ui: &mut Ui,
    icons: &Icons,
    trigger: &Response,
    items: &[(&str, &str, bool, bool)],
) -> Option<usize> {
    let mut picked = None;
    Popup::menu(trigger)
        .frame(popup_frame())
        .width(190.0)
        .gap(POPUP_GAP)
        .align(egui::RectAlign::BOTTOM_END)
        .align_alternatives(&[egui::RectAlign::TOP_END])
        .show(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for (i, (icon, label, danger, sep)) in items.iter().enumerate() {
                if *sep {
                    let w = ui.available_width();
                    let (r, _) = ui.allocate_exact_size(vec2(w, 7.0), Sense::hover());
                    ui.painter().line_segment(
                        [pos2(r.left() + 4.0, r.center().y), pos2(r.right() - 4.0, r.center().y)],
                        Stroke::new(1.0, BORDER_SUBTLE),
                    );
                }
                let w = ui.available_width();
                let (rect, resp) = ui.allocate_exact_size(vec2(w, 34.0), Sense::click());
                let hovered = resp.hovered();
                let (bg, fg) = match (danger, hovered) {
                    (true, true) => (alpha(DANGER, 0.12), DANGER_TEXT),
                    (true, false) => (Color32::TRANSPARENT, DANGER_TEXT),
                    (false, true) => (BG_SURFACE_HOVER, TEXT_WHITE),
                    (false, false) => (Color32::TRANSPARENT, TEXT_SECONDARY),
                };
                ui.painter().rect_filled(rect, 6, bg);
                let ir = Rect::from_center_size(pos2(rect.left() + 12.0 + 7.5, rect.center().y), Vec2::splat(15.0));
                icons.paint(ui.painter(), icon, ir, fg);
                let g = galley(ui, label, 12.5, Weight::Medium, fg);
                ui.painter()
                    .galley(pos2(rect.left() + 12.0 + 15.0 + 9.0, rect.center().y - g.size().y / 2.0), g, fg);
                if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    picked = Some(i);
                }
            }
        });
    picked
}

/* ------------------------------------------------------------------------- */
/* Date picker                                                               */
/* ------------------------------------------------------------------------- */

pub enum DateAction {
    Picked(NaiveDate),
    Cleared,
}

/// Read-only date field (DD-MMM-YYYY) with a calendar popup.
/// `clear_label`: label for the second footer button ("Close" or "Clear").
pub fn date_field(
    ui: &mut Ui,
    icons: &Icons,
    id_salt: impl egui::AsIdSalt,
    value: Option<NaiveDate>,
    clear_label: &str,
) -> Option<DateAction> {
    let id = ui.make_persistent_id(id_salt);
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(width, INPUT_HEIGHT), Sense::click());
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
    let popup_id = id.with("cal");
    // Design-review mode (JHA_REVIEW_OPEN=date): open the first calendar once.
    if std::env::var("JHA_REVIEW_OPEN").as_deref() == Ok("date") {
        let flag = egui::Id::unique("review-open-date");
        if !ui.ctx().data(|d| d.get_temp::<bool>(flag).unwrap_or(false)) {
            ui.ctx().data_mut(|d| d.insert_temp(flag, true));
            Popup::open_id(ui.ctx(), popup_id);
        }
    }
    let open = Popup::is_id_open(ui.ctx(), popup_id);

    let (bg, border) = if open {
        (BG_SURFACE, BRAND)
    } else if resp.hovered() {
        (BG_INPUT_HOVER, BORDER_LIGHT)
    } else {
        (BG_SURFACE, BORDER_SUBTLE)
    };
    if open {
        ui.painter()
            .rect_stroke(rect.expand(3.0), 11, Stroke::new(3.0, alpha(BRAND, 0.15)), StrokeKind::Inside);
    }
    ui.painter().rect(rect, 8, bg, Stroke::new(1.0, border), StrokeKind::Inside);
    let (label, color) = match value {
        Some(d) => (dates::display(d), TEXT_PRIMARY),
        None => ("DD-MMM-YYYY".to_string(), TEXT_SUBTLE),
    };
    let g = galley(ui, &label, 13.5, Weight::Regular, color);
    ui.painter()
        .galley(pos2(rect.left() + 14.0, rect.center().y - g.size().y / 2.0), g, color);
    let ir = Rect::from_center_size(pos2(rect.right() - 14.0 - 8.0, rect.center().y), Vec2::splat(16.0));
    icons.paint(ui.painter(), "calendar-blank", ir, if open { BRAND } else { TEXT_DIM });

    // Month being displayed in the popup.
    let view_id = id.with("view");
    if resp.clicked() {
        let d = value.unwrap_or_else(dates::today);
        ui.data_mut(|m| m.insert_temp(view_id, (d.year(), d.month())));
    }

    let mut action = None;
    Popup::from_toggle_button_response(&resp)
        .id(popup_id)
        .width(292.0)
        // Right-aligned under the field, with a gap (above it when there's no room below).
        .gap(POPUP_GAP)
        .align(egui::RectAlign::BOTTOM_END)
        .align_alternatives(&[egui::RectAlign::TOP_END])
        .frame(popup_frame().inner_margin(Margin::same(12)))
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            let (mut y, mut m) = ui
                .data(|d| d.get_temp::<(i32, u32)>(view_id))
                .unwrap_or_else(|| (dates::today().year(), dates::today().month()));
            ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

            // Header
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                if icon_button(ui, icons, "caret-left", Vec2::splat(30.0), 14.0, TEXT_SECONDARY, BG_SURFACE_HOVER, None).clicked() {
                    if m == 1 {
                        m = 12;
                        y -= 1;
                    } else {
                        m -= 1;
                    }
                }
                let title = format!("{} {}", dates::MONTH_LONG[(m - 1) as usize], y);
                let w = ui.available_width() - 30.0;
                let (r, _) = ui.allocate_exact_size(vec2(w, 30.0), Sense::hover());
                ui.painter().text(r.center(), Align2::CENTER_CENTER, title, font(13.5, Weight::SemiBold), TEXT_WHITE);
                if icon_button(ui, icons, "caret-right", Vec2::splat(30.0), 14.0, TEXT_SECONDARY, BG_SURFACE_HOVER, None).clicked() {
                    if m == 12 {
                        m = 1;
                        y += 1;
                    } else {
                        m += 1;
                    }
                }
            });
            ui.data_mut(|d| d.insert_temp(view_id, (y, m)));

            let cell = vec2(268.0 / 7.0, 32.0);
            // Weekdays
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for wd in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
                    let (r, _) = ui.allocate_exact_size(vec2(cell.x, 22.0), Sense::hover());
                    ui.painter().text(r.center(), Align2::CENTER_CENTER, wd, font(11.0, Weight::SemiBold), TEXT_DIM);
                }
            });

            // Day grid
            let first = NaiveDate::from_ymd_opt(y, m, 1).unwrap_or_else(dates::today);
            let lead = first.weekday().num_days_from_sunday() as i64;
            let days = dates::days_in_month(y, m) as i64;
            let total = if lead + days > 35 { 42 } else { 35 };
            let start = first - chrono::Duration::days(lead);
            let today = dates::today();
            for row in 0..(total / 7) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    for col in 0..7 {
                        let d = start + chrono::Duration::days(row * 7 + col);
                        let other = d.month() != m;
                        let (r, resp) = ui.allocate_exact_size(cell, Sense::click());
                        let r = r.shrink2(vec2(2.0, 1.0));
                        let selected = Some(d) == value;
                        let painter = ui.painter();
                        if selected {
                            painter.rect_filled(r, 6, BRAND);
                        } else if resp.hovered() {
                            painter.rect_filled(r, 6, BG_SURFACE_HOVER);
                        }
                        if d == today && !selected {
                            painter.rect_stroke(r, 6, Stroke::new(1.0, alpha(BRAND, 0.7)), StrokeKind::Inside);
                        }
                        let c = if selected {
                            TEXT_WHITE
                        } else if other {
                            TEXT_SUBTLE
                        } else {
                            TEXT_SECONDARY
                        };
                        painter.text(
                            r.center(),
                            Align2::CENTER_CENTER,
                            d.day().to_string(),
                            font(12.5, if selected { Weight::SemiBold } else { Weight::Regular }),
                            c,
                        );
                        if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                            action = Some(DateAction::Picked(d));
                            ui.close();
                        }
                    }
                });
            }

            ui.add_space(4.0);
            let w = ui.available_width();
            let (r, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
            ui.painter().rect_filled(r, 0, BORDER_SUBTLE);
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let half = (ui.available_width() - 8.0) / 2.0;
                if Btn::new(Kind::Secondary, None, "Today").height(32.0).width(half).show(ui, icons).clicked() {
                    action = Some(DateAction::Picked(dates::today()));
                    ui.close();
                }
                if Btn::new(Kind::Tertiary, None, clear_label).height(32.0).width(half).show(ui, icons).clicked() {
                    if clear_label == "Clear" {
                        action = Some(DateAction::Cleared);
                    }
                    ui.close();
                }
            });
        });
    action
}

/* ------------------------------------------------------------------------- */
/* Cards & misc                                                              */
/* ------------------------------------------------------------------------- */

pub fn card(fill: Color32, border: Color32, radius: u8, margin: Margin) -> Frame {
    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(radius)
        .inner_margin(margin)
}

/// Small rounded pill/badge.
pub fn pill(ui: &mut Ui, s: &str, fg: Color32, bg: Color32, border: Option<Color32>, size: f32) -> Response {
    let g = galley(ui, s, size, Weight::SemiBold, fg);
    let pad = vec2(8.0, 3.0);
    let (rect, resp) = ui.allocate_exact_size(g.size() + pad * 2.0, Sense::hover());
    ui.painter().rect(
        rect,
        12,
        bg,
        border.map(|b| Stroke::new(1.0, b)).unwrap_or(Stroke::NONE),
        StrokeKind::Inside,
    );
    ui.painter().galley(rect.min + pad, g, fg);
    resp
}

pub fn hline(ui: &mut Ui, color: Color32) {
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
    ui.painter().rect_filled(r, 0, color);
}

/* ------------------------------------------------------------------------- */
/* Toasts                                                                    */
/* ------------------------------------------------------------------------- */

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warn,
    Error,
}

pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub born: f64,
}

pub fn show_toasts(ctx: &egui::Context, icons: &Icons, toasts: &mut Vec<Toast>) {
    let now = ctx.input(|i| i.time);
    for t in toasts.iter_mut() {
        if t.born < 0.0 {
            t.born = now;
        }
    }
    toasts.retain(|t| now - t.born < 3.2);
    if toasts.is_empty() {
        return;
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(100));

    // Toasts are painted directly (exact sizes, newest at the bottom-right).
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, Id::unique("toasts")));
    let screen = ctx.content_rect();
    let mut bottom = screen.bottom() - 24.0;
    for t in toasts.iter().rev() {
        let age = (now - t.born) as f32;
        let fade = ((3.2 - age) / 0.25).clamp(0.0, 1.0) * (age / 0.15).clamp(0.0, 1.0);
        let slide = (1.0 - (age / 0.2).clamp(0.0, 1.0)) * 12.0;
        let (accent, icon) = match t.kind {
            ToastKind::Info => (BRAND, "info"),
            ToastKind::Success => (SUCCESS, "check-circle"),
            ToastKind::Warn => (WARNING, "warning-circle"),
            ToastKind::Error => (DANGER, "x-circle"),
        };
        let mut job = LayoutJob::single_section(
            t.message.clone(),
            egui::TextFormat {
                font_id: font(13.0, Weight::Medium),
                color: alpha(TEXT_PRIMARY, fade),
                ..Default::default()
            },
        );
        job.wrap.max_width = 400.0;
        let g = painter.layout_job(job);
        let size = vec2(16.0 + 16.0 + 10.0 + g.size().x + 16.0, (g.size().y.max(16.0)) + 20.0);
        let rect = Rect::from_min_size(
            pos2(screen.right() - 24.0 - size.x, bottom - size.y + slide),
            size,
        );
        painter.add(
            Shadow {
                offset: [0, 10],
                blur: 28,
                spread: 0,
                color: Color32::from_black_alpha((128.0 * fade) as u8),
            }
            .as_shape(rect, CornerRadius::same(8)),
        );
        painter.rect(
            rect,
            8,
            alpha(BG_ELEVATED, 0.97 * fade),
            Stroke::new(1.0, alpha(TEXT_WHITE, 0.12 * fade)),
            StrokeKind::Inside,
        );
        painter.rect_filled(
            Rect::from_min_max(rect.min, pos2(rect.min.x + 3.0, rect.max.y)),
            CornerRadius { nw: 8, sw: 8, ne: 0, se: 0 },
            alpha(accent, fade),
        );
        let ir = Rect::from_min_size(pos2(rect.left() + 16.0, rect.center().y - 8.0), Vec2::splat(16.0));
        icons.paint(&painter, icon, ir, alpha(accent, fade));
        let tp = pos2(ir.right() + 10.0, rect.center().y - g.size().y / 2.0);
        painter.galley(tp, g, TEXT_PRIMARY);
        bottom = rect.top() - slide - 8.0;
    }
}

pub fn shadow_behind(painter: &egui::Painter, rect: Rect) {
    painter.add(
        Shadow {
            offset: [0, 4],
            blur: 30,
            spread: 0,
            color: Color32::from_black_alpha(128),
        }
        .as_shape(rect, CornerRadius::ZERO),
    );
}
