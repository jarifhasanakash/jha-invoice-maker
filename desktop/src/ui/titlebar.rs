//! Custom 36 px window title bar (the window is created without system
//! decorations): app icon and name, drag to move, double-click to maximize,
//! minimize / maximize-restore / close buttons, and resize handles on the
//! window edges.

use eframe::egui::{
    self, Align2, Color32, CursorIcon, Id, Rect, Sense, Stroke, Ui, ViewportCommand, pos2, vec2,
};
use egui::viewport::ResizeDirection;

use super::theme::*;
use crate::fonts::Weight;

pub const HEIGHT: f32 = 36.0;
const BUTTON_W: f32 = 46.0;
const CLOSE_HOVER: Color32 = Color32::from_rgb(0xc4, 0x2b, 0x1c);
const BUTTON_HOVER: Color32 = Color32::from_rgb(0x2d, 0x2d, 0x2d);
const BUTTON_PRESS: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);

#[derive(Copy, Clone)]
enum Caption {
    Minimize,
    Maximize,
    Restore,
    Close,
}

/// Draws the title bar into `ui` (a 36 px top panel).
pub fn show(ui: &mut Ui, icon: Option<&egui::TextureHandle>, title: &str) {
    let ctx = ui.ctx().clone();
    let rect = ui.max_rect();
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    let focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
    let painter = ui.painter().clone();

    painter.rect_filled(rect, 0.0, BG_APP);
    painter.line_segment(
        [pos2(rect.left(), rect.bottom() - 0.5), pos2(rect.right(), rect.bottom() - 0.5)],
        Stroke::new(1.0, BG_SURFACE_HOVER),
    );

    // Caption buttons (right to left: close, maximize/restore, minimize).
    let buttons = [
        Caption::Close,
        if maximized { Caption::Restore } else { Caption::Maximize },
        Caption::Minimize,
    ];
    let mut right = rect.right();
    for kind in buttons {
        let r = Rect::from_min_max(pos2(right - BUTTON_W, rect.top()), pos2(right, rect.bottom() - 1.0));
        right -= BUTTON_W;
        let id = Id::unique(("titlebar-btn", kind as u8));
        let resp = ui.interact(r, id, Sense::click());
        let (bg, fg) = match (kind, resp.is_pointer_button_down_on(), resp.hovered()) {
            (Caption::Close, true, _) => (Some(alpha(CLOSE_HOVER, 0.8)), TEXT_WHITE),
            (Caption::Close, false, true) => (Some(CLOSE_HOVER), TEXT_WHITE),
            (_, true, _) => (Some(BUTTON_PRESS), TEXT_WHITE),
            (_, false, true) => (Some(BUTTON_HOVER), TEXT_WHITE),
            _ => (None, if focused { TEXT_SECONDARY } else { TEXT_SUBTLE }),
        };
        if let Some(bg) = bg {
            painter.rect_filled(r, 0.0, bg);
        }
        paint_glyph(&painter, kind, r.center(), fg, ctx.pixels_per_point());
        let tip = match kind {
            Caption::Minimize => "Minimize",
            Caption::Maximize => "Maximize",
            Caption::Restore => "Restore Down",
            Caption::Close => "Close",
        };
        if resp.on_hover_text(tip).clicked() {
            ctx.send_viewport_cmd(match kind {
                Caption::Minimize => ViewportCommand::Minimized(true),
                Caption::Maximize => ViewportCommand::Maximized(true),
                Caption::Restore => ViewportCommand::Maximized(false),
                Caption::Close => ViewportCommand::Close,
            });
        }
    }

    // Drag area: everything left of the buttons.
    let drag_rect = Rect::from_min_max(rect.min, pos2(right, rect.bottom()));
    let drag = ui.interact(drag_rect, Id::unique("titlebar-drag"), Sense::click_and_drag());
    if drag.double_clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    } else if drag.drag_started_by(egui::PointerButton::Primary) {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }

    // Icon + title.
    let mut x = rect.left() + 12.0;
    if let Some(tex) = icon {
        let size = 18.0;
        let r = Rect::from_center_size(pos2(x + size / 2.0, rect.center().y), vec2(size, size));
        painter.image(tex.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        x += size + 10.0;
    }
    painter.text(
        pos2(x, rect.center().y),
        Align2::LEFT_CENTER,
        title,
        font(12.5, Weight::Medium),
        if focused { TEXT_SECONDARY } else { TEXT_SUBTLE },
    );
}

fn paint_glyph(painter: &egui::Painter, kind: Caption, c: egui::Pos2, color: Color32, ppp: f32) {
    // Snap to the pixel grid so 1 px lines stay crisp.
    let snap = |v: f32| (v * ppp).round() / ppp + 0.5 / ppp;
    let c = pos2(snap(c.x), snap(c.y));
    let s = Stroke::new(1.0, color);
    let h = 5.0;
    match kind {
        Caption::Minimize => {
            painter.line_segment([pos2(c.x - h, c.y), pos2(c.x + h, c.y)], s);
        }
        Caption::Maximize => {
            painter.rect_stroke(
                Rect::from_center_size(c, vec2(2.0 * h, 2.0 * h)),
                1.0,
                s,
                egui::StrokeKind::Middle,
            );
        }
        Caption::Restore => {
            let front = Rect::from_min_size(pos2(c.x - h, c.y - h + 2.0), vec2(2.0 * h - 2.0, 2.0 * h - 2.0));
            painter.rect_stroke(front, 1.0, s, egui::StrokeKind::Middle);
            // Back window: top and right edges only.
            painter.line_segment([pos2(c.x - h + 2.0, c.y - h), pos2(c.x + h, c.y - h)], s);
            painter.line_segment([pos2(c.x + h, c.y - h), pos2(c.x + h, c.y + h - 2.0)], s);
        }
        Caption::Close => {
            painter.line_segment([pos2(c.x - h, c.y - h), pos2(c.x + h, c.y + h)], s);
            painter.line_segment([pos2(c.x + h, c.y - h), pos2(c.x - h, c.y + h)], s);
        }
    }
}

/// Resize handles along the window edges (the borderless window has no
/// system frame to grab). Call after all panels so the handles sit on top.
pub fn resize_handles(ctx: &egui::Context) {
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    let fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
    if maximized || fullscreen {
        return;
    }
    let screen = ctx.content_rect();
    let edge = 5.0;
    let corner = 12.0;
    let (l, r, t, b) = (screen.left(), screen.right(), screen.top(), screen.bottom());
    let zones = [
        (ResizeDirection::NorthWest, Rect::from_min_max(pos2(l, t), pos2(l + corner, t + corner)), CursorIcon::ResizeNorthWest),
        (ResizeDirection::NorthEast, Rect::from_min_max(pos2(r - corner, t), pos2(r, t + corner)), CursorIcon::ResizeNorthEast),
        (ResizeDirection::SouthWest, Rect::from_min_max(pos2(l, b - corner), pos2(l + corner, b)), CursorIcon::ResizeSouthWest),
        (ResizeDirection::SouthEast, Rect::from_min_max(pos2(r - corner, b - corner), pos2(r, b)), CursorIcon::ResizeSouthEast),
        (ResizeDirection::North, Rect::from_min_max(pos2(l + corner, t), pos2(r - corner, t + edge)), CursorIcon::ResizeVertical),
        (ResizeDirection::South, Rect::from_min_max(pos2(l + corner, b - edge), pos2(r - corner, b)), CursorIcon::ResizeVertical),
        (ResizeDirection::West, Rect::from_min_max(pos2(l, t + corner), pos2(l + edge, b - corner)), CursorIcon::ResizeHorizontal),
        (ResizeDirection::East, Rect::from_min_max(pos2(r - edge, t + corner), pos2(r, b - corner)), CursorIcon::ResizeHorizontal),
    ];
    egui::Area::new(Id::unique("window-resize-handles"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .interactable(true)
        .show(ctx, |ui| {
            for (i, (dir, rect, cursor)) in zones.into_iter().enumerate() {
                let resp = ui.interact(rect, Id::unique(("window-resize", i)), Sense::drag());
                if resp.hovered() || resp.dragged() {
                    ctx.set_cursor_icon(cursor);
                }
                if resp.drag_started_by(egui::PointerButton::Primary) {
                    ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
                }
            }
        });
}
