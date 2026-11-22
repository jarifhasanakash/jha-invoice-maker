//! Invoice editor: sidebar form and live preview.

use eframe::egui::{
    self, Align2, Color32, CursorIcon, Frame, Margin, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
    pos2, vec2,
};

use super::theme::*;
use super::widgets::*;
use super::{Action, App};
use crate::fonts::Weight;
use crate::layout;
use crate::model::{self, LineItem};

const ZOOM_MIN: f32 = 0.4;
const ZOOM_MAX: f32 = 1.8;

impl App {
    /* ---------------- Sidebar ---------------- */

    pub(super) fn sidebar(&mut self, ui: &mut Ui) {
        let now = ui.input(|i| i.time);
        if self.ed.flash_client > now || self.ed.flash_items > now {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(200));
        }
        let mut sa = egui::ScrollArea::vertical();
        if let Some(y) = self.review_scroll.take() {
            sa = sa.vertical_scroll_offset(y);
        }
        sa
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(12.0);
                self.section(ui, true, |app, ui| app.details_section(ui));
                self.section(ui, false, |app, ui| app.client_section(ui, now));
                self.section(ui, false, |app, ui| app.items_section(ui, now));
                self.section(ui, false, |app, ui| app.breakdown_section(ui));
                self.section(ui, false, |app, ui| app.note_section(ui));
                ui.add_space(4.0);
            });
    }

    fn section(&mut self, ui: &mut Ui, first: bool, f: impl FnOnce(&mut Self, &mut Ui)) {
        if !first {
            let w = ui.available_width();
            let (r, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
            ui.painter().rect_filled(r, 0, BORDER_SUBTLE);
        }
        Frame::new().inner_margin(Margin::same(20)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 16.0;
            f(self, ui);
        });
    }

    fn details_section(&mut self, ui: &mut Ui) {
        section_heading(ui, None, "Invoice Details");
        let icons = &self.icons;
        let ed = &mut self.ed;
        let mut picked = None;
        two_columns(ui, |a, b| {
            a.spacing_mut().item_spacing.y = 6.0;
            field_label(a, "Invoice Number", false);
            input(a, &mut ed.number, "0001", false);
            b.spacing_mut().item_spacing.y = 6.0;
            field_label(b, "Date", false);
            picked = date_field(b, icons, "invoice-date", Some(ed.date), "Close");
        });
        if let Some(DateAction::Picked(d)) = picked {
            self.ed.date = d;
        }
    }

    fn client_section(&mut self, ui: &mut Ui, now: f64) {
        section_heading(ui, None, "Client");
        let mut options = vec!["Select a Client".to_string()];
        let mut ids = vec![None];
        for c in &self.db.clients {
            options.push(if c.location.is_empty() {
                c.name.clone()
            } else {
                format!("{} ({})", c.name, c.location)
            });
            ids.push(Some(c.id.clone()));
        }
        let selected = match &self.ed.client.id {
            Some(id) => ids.iter().position(|x| x.as_deref() == Some(id.as_str())),
            None => None,
        };
        let w = ui.available_width();
        if let Some(i) = select(
            ui,
            &self.icons,
            "client-select",
            &options,
            selected,
            "Select a Client",
            w,
            42.0,
            None,
            self.ed.flash_client > now,
        ) {
            self.actions.push(Action::SelectClient(ids[i].clone()));
        }
        if self.db.clients.is_empty() {
            ui.horizontal(|ui| {
                text(ui, "No saved clients yet.", 12.0, Weight::Regular, TEXT_DIM);
                let r = ui.add(
                    egui::Label::new(egui::RichText::new("Add a client").font(font(12.0, Weight::Medium)).color(BRAND))
                        .sense(Sense::click()),
                );
                if r.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    self.clients.form = Some(super::ClientForm {
                        currency: "USD".into(),
                        ..Default::default()
                    });
                    self.actions.push(Action::SwitchTab(super::Tab::Clients));
                }
            });
        }
    }

    fn items_section(&mut self, ui: &mut Ui, now: f64) {
        let n = self.ed.items.len();
        tight_row(ui, |ui| {
            section_heading(ui, None, "Line Items");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                pill(
                    ui,
                    &format!("{n} {}", if n == 1 { "item" } else { "items" }),
                    TEXT_DIM,
                    BG_SURFACE_HOVER,
                    None,
                    11.0,
                );
            });
        });

        let flash = self.ed.flash_items > now;
        let mut remove = None;
        let mut modified = false;
        for i in 0..self.ed.items.len() {
            ui.push_id(("item", i), |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                let item = &mut self.ed.items[i];
                if textarea(ui, &mut item.description, "Description", 1, 13.5, TEXT_PRIMARY, flash && i == 0).changed() {
                    modified = true;
                }
                if textarea(ui, &mut item.sub, "Sub-detail (Line breaks supported)", 3, 12.5, TEXT_MUTED, false).changed() {
                    modified = true;
                }
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let w = ui.available_width() - INPUT_HEIGHT - 8.0;
                    let price_text = &mut self.ed.price_text[i];
                    let r = ui.add_sized(
                        vec2(w, INPUT_HEIGHT),
                        egui::TextEdit::singleline(price_text)
                            .hint_text(egui::RichText::new("Price").color(TEXT_SUBTLE))
                            .margin(Margin::symmetric(14, 11))
                            .font(font(13.5, Weight::Regular)),
                    );
                    if r.changed() {
                        let cleaned: String = price_text
                            .chars()
                            .filter(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | 'e' | 'E' | '+'))
                            .collect();
                        if cleaned != *price_text {
                            *price_text = cleaned;
                        }
                        self.ed.items[i].price = model::parse_float_prefix(price_text);
                        modified = true;
                    }
                    if r.lost_focus() && self.ed.price_text[i].trim().is_empty() {
                        self.ed.price_text[i] = "0".into();
                    }
                    let resp = icon_button(
                        ui,
                        &self.icons,
                        "x",
                        Vec2::splat(INPUT_HEIGHT),
                        16.0,
                        DANGER,
                        alpha(DANGER, 0.12),
                        Some(N720),
                    )
                    .on_hover_text("Remove line item");
                    if resp.clicked() {
                        remove = Some(i);
                    }
                });
                ui.add_space(8.0);
                hline(ui, BORDER_SUBTLE);
            });
        }
        if let Some(i) = remove {
            self.ed.items.remove(i);
            self.ed.price_text.remove(i);
            if self.ed.items.is_empty() {
                self.ed.items.push(LineItem::new("", "", 0.0));
                self.ed.price_text.push("0".into());
            }
            modified = true;
        }
        if Btn::new(Kind::Secondary, Some("plus"), "Add Item")
            .full(ui)
            .show(ui, &self.icons)
            .clicked()
        {
            self.ed.items.push(LineItem::new("New Item", "", 0.0));
            self.ed.price_text.push("0".into());
            modified = true;
        }
        if modified {
            self.ed.user_modified = true;
        }
    }

    fn toggle_card(ui: &mut Ui, label: &str, on: &mut bool, add: impl FnOnce(&mut Ui)) {
        let id = ui.scope_id().with(label);
        let hovered = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
        let resp = Frame::new()
            .fill(if hovered { BG_SURFACE_HOVER } else { BG_SURFACE })
            .stroke(Stroke::new(1.0, if hovered { N720 } else { BORDER_SUBTLE }))
            .corner_radius(7)
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().interact_size.y = 22.0;
                let row = ui.horizontal(|ui| {
                    let lr = ui.add(
                        egui::Label::new(
                            egui::RichText::new(label)
                                .font(font(13.0, Weight::Medium))
                                .color(if hovered { TEXT_WHITE } else { TEXT_SECONDARY }),
                        )
                        .sense(Sense::click()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        switch(ui, on);
                    });
                    lr
                });
                if row.inner.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    *on = !*on;
                }
                if *on {
                    add(ui);
                }
            })
            .response;
        let h = resp.hovered() || ui.rect_contains_pointer(resp.rect);
        ui.ctx().data_mut(|d| d.insert_temp(id, h));
    }

    fn breakdown_section(&mut self, ui: &mut Ui) {
        section_heading(ui, None, "Payment Breakdown");
        ui.spacing_mut().item_spacing.y = 10.0;
        let ed = &mut self.ed;
        let mut advance_text = std::mem::take(&mut ed.advance_text);
        Self::toggle_card(ui, "Include advance payment", &mut ed.toggle_advance, |ui| {
            ui.add_space(4.0);
            hline(ui, BORDER_SUBTLE);
            ui.add_space(4.0);
            ui.spacing_mut().item_spacing.y = 6.0;
            field_label(ui, "Advance Amount", false);
            let r = input(ui, &mut advance_text, "00", false);
            if r.changed() {
                advance_text.retain(|c| c.is_ascii_digit() || matches!(c, '.' | '-'));
            }
        });
        ed.advance_text = advance_text;
        Self::toggle_card(ui, "Include balance due", &mut ed.toggle_balance, |_| {});
    }

    fn note_section(&mut self, ui: &mut Ui) {
        section_heading(ui, None, "Invoice Note");
        textarea(ui, &mut self.ed.note, model::DEFAULT_NOTE, 2, 13.5, TEXT_PRIMARY, false);
    }

    /* ---------------- Preview ---------------- */

    pub(super) fn preview_area(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let ppp = ctx.pixels_per_point();
        let avail = ui.available_rect_before_wrap();

        if self.preview.auto_fit {
            let available = avail.width() - 80.0;
            self.preview.zoom = if available < layout::PAPER_WIDTH {
                ((available / layout::PAPER_WIDTH) * 0.98).max(ZOOM_MIN)
            } else {
                1.0
            };
        }

        // Ctrl + mouse wheel: zoom around the mouse pointer (the point under the
        // cursor stays put), instead of growing from the top-left corner.
        let mut new_offset = None;
        let (zoom_delta, pointer) = ui.input(|i| (i.zoom_delta(), i.pointer.hover_pos()));
        if (zoom_delta - 1.0).abs() > 1e-3 {
            if let (Some(p), Some(old)) = (pointer.filter(|p| avail.contains(*p)), self.preview.last_paper) {
                let old_zoom = self.preview.zoom;
                let nz = (old_zoom * zoom_delta).clamp(ZOOM_MIN, ZOOM_MAX);
                if (nz - old_zoom).abs() > 1e-4 && old.width() > 0.0 && old.height() > 0.0 {
                    let ux = (p.x - old.min.x) / old.width();
                    let uy = (p.y - old.min.y) / old.height();
                    self.preview.auto_fit = false;
                    self.preview.zoom = nz;
                    let ps = self.preview.page_size * nz;
                    let content_w = (ps.x + 80.0).max(avail.width());
                    let paper_x = content_w / 2.0 - ps.x / 2.0;
                    let paper_y = 24.0 + 44.0 + 24.0;
                    let off_x = paper_x + ux * ps.x - (p.x - avail.min.x);
                    let off_y = paper_y + uy * ps.y - (p.y - avail.min.y);
                    new_offset = Some(vec2(off_x.max(0.0), off_y.max(0.0)));
                }
            }
        }

        // Re-render the page only when its content or resolution changes.
        let view = self.editor_view();
        let zoom = self.preview.zoom;
        let max_dim = 8192.0;
        // Render at exactly the device-pixel scale it is displayed at, so the
        // bitmap maps 1:1 onto the screen (no resampling blur).
        let mut scale = zoom * ppp;
        let logo_key = std::sync::Arc::as_ptr(&self.logo) as *const u8 as usize;
        let needs = match &self.preview.rendered {
            Some((v, s, l)) => *v != view || (*s - scale).abs() > 1e-4 || *l != logo_key,
            None => true,
        };
        if needs {
            let page = layout::layout(&view, self.logo.clone());
            scale = scale.min(max_dim / page.height.max(page.width));
            let (px, size) = self.preview.rasterizer.render(&page, scale);
            let img = egui::ColorImage::from_rgba_premultiplied(size, &px);
            match &mut self.preview.texture {
                Some(t) => t.set(img, egui::TextureOptions::LINEAR),
                None => {
                    self.preview.texture =
                        Some(ctx.load_texture("invoice-preview", img, egui::TextureOptions::LINEAR))
                }
            }
            self.preview.page_size = vec2(page.width, page.height);
            self.preview.rendered = Some((view, zoom * ppp, logo_key));
        }

        // Paper inside a scroll area, toolbar floating on top.
        // Display size taken from the texture's pixel size, so it is exactly 1:1.
        let paper = match &self.preview.texture {
            Some(t) => {
                let [w, h] = t.size();
                let s = (self.preview.page_size.x * zoom) / (w as f32 / ppp);
                // s≈1 normally; differs only when the page was capped at max_dim.
                vec2(w as f32, h as f32) / ppp * s.max(1.0)
            }
            None => self.preview.page_size * zoom,
        };
        let toolbar_h = 44.0;
        let mut scroll = egui::ScrollArea::both().id_salt("preview-scroll").auto_shrink([false, false]);
        if let Some(o) = new_offset {
            scroll = scroll.scroll_offset(o);
        }
        let mut shown_paper = None;
        scroll.show(ui, |ui| {
                let content_w = (paper.x + 80.0).max(ui.available_width());
                let (rect, _) = ui.allocate_exact_size(
                    vec2(content_w, 24.0 + toolbar_h + 24.0 + paper.y + 60.0),
                    Sense::hover(),
                );
                // Snap to whole device pixels to keep the page crisp.
                let x = ((rect.center().x - paper.x / 2.0) * ppp).round() / ppp;
                let y = ((rect.top() + 24.0 + toolbar_h + 24.0) * ppp).round() / ppp;
                let paper_rect = Rect::from_min_size(pos2(x, y), paper);
                shown_paper = Some(paper_rect);
                shadow_behind(ui.painter(), paper_rect);
                if let Some(tex) = &self.preview.texture {
                    ui.painter().image(
                        tex.id(),
                        paper_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
            });
        self.preview.last_paper = shown_paper;

        // Floating toolbar (sticky, like the original).
        let tb_w = (avail.width() - 80.0).min(820.0).max(320.0);
        let tb_rect = Rect::from_min_size(pos2(avail.center().x - tb_w / 2.0, avail.top() + 10.0), vec2(tb_w, toolbar_h));
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(tb_rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
        self.preview_toolbar(&mut child, tb_rect);
    }

    fn preview_toolbar(&mut self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter().clone();
        painter.add(
            egui::Shadow {
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: Color32::from_black_alpha(100),
            }
            .as_shape(rect, egui::CornerRadius::same(22)),
        );
        painter.rect(
            rect,
            22,
            alpha(BG_ELEVATED, 0.95),
            Stroke::new(1.0, alpha(TEXT_WHITE, 0.12)),
            StrokeKind::Inside,
        );
        // Block clicks from reaching the page beneath.
        ui.interact(rect, ui.scope_id().with("toolbar-bg"), Sense::click());

        ui.add_space(16.0);
        ui.spacing_mut().item_spacing.x = 10.0;
        let mut job = egui::text::LayoutJob::default();
        job.append(
            "PREVIEW",
            0.0,
            egui::TextFormat {
                font_id: font(11.0, Weight::SemiBold),
                color: TEXT_DIM,
                extra_letter_spacing: 0.8,
                ..Default::default()
            },
        );
        ui.label(job);
        self.icons.show(ui, "receipt", 14.0, TEXT_SECONDARY);
        let number = if self.ed.number.trim().is_empty() { "0001".to_string() } else { self.ed.number.trim().to_string() };
        text(ui, &format!("Invoice #{number}"), 13.0, Weight::Medium, TEXT_PRIMARY);

        if self.ed.editing_id.is_some() {
            // "Editing" chip with an exit button.
            let g = galley(ui, "Editing", 10.5, Weight::SemiBold, BRAND);
            let size = vec2(12.0 + g.size().x + 4.0 + 20.0, 22.0);
            let (r, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter().rect(r, 11, alpha(BRAND, 0.14), Stroke::new(1.0, alpha(BRAND, 0.35)), StrokeKind::Inside);
            ui.painter().galley(pos2(r.left() + 12.0, r.center().y - g.size().y / 2.0), g, BRAND);
            let xr = Rect::from_center_size(pos2(r.right() - 11.0, r.center().y), Vec2::splat(18.0));
            let xresp = ui
                .interact(xr, ui.scope_id().with("exit-edit"), Sense::click())
                .on_hover_text("Exit edit mode & start a fresh invoice")
                .on_hover_cursor(CursorIcon::PointingHand);
            if xresp.hovered() {
                ui.painter().circle_filled(xr.center(), 9.0, alpha(BRAND, 0.25));
            }
            self.icons.paint(ui.painter(), "x", xr.shrink(4.0), BRAND);
            if xresp.clicked() {
                self.actions.push(Action::NewInvoice { silent: false });
            }
            pill(ui, &self.ed.status, status_color(&self.ed.status), alpha(TEXT_WHITE, 0.06), None, 10.5);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(5.0);
            ui.spacing_mut().item_spacing.x = 2.0;
            let zoom_btn = |ui: &mut Ui, icons: &Icons, icon: &str, label: &str| -> egui::Response {
                let g = galley(ui, label, 12.0, Weight::Medium, TEXT_SECONDARY);
                let w = if label.is_empty() { 34.0 } else { 34.0 + g.size().x + 4.0 };
                let (r, resp) = ui.allocate_exact_size(vec2(w, 34.0), Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(r, 17, alpha(TEXT_WHITE, 0.08));
                }
                let c = if resp.hovered() { TEXT_WHITE } else { TEXT_SECONDARY };
                let ir = Rect::from_center_size(pos2(r.left() + 17.0, r.center().y), Vec2::splat(14.0));
                icons.paint(ui.painter(), icon, ir, c);
                if !label.is_empty() {
                    let g = galley(ui, label, 12.0, Weight::Medium, c);
                    ui.painter().galley(pos2(ir.right() + 4.0, r.center().y - g.size().y / 2.0), g, c);
                }
                resp.on_hover_cursor(CursorIcon::PointingHand)
            };
            if zoom_btn(ui, &self.icons, "corners-out", "Fit").on_hover_text("Fit to screen").clicked() {
                self.preview.auto_fit = true;
            }
            if zoom_btn(ui, &self.icons, "plus", "").on_hover_text("Zoom in").clicked() {
                self.preview.auto_fit = false;
                self.preview.zoom = (self.preview.zoom + 0.1).clamp(ZOOM_MIN, ZOOM_MAX);
            }
            let (r, resp) = ui.allocate_exact_size(vec2(46.0, 34.0), Sense::click());
            let resp = resp
                .on_hover_text("Click to reset zoom (100%)")
                .on_hover_cursor(CursorIcon::PointingHand);
            ui.painter().text(
                r.center(),
                Align2::CENTER_CENTER,
                format!("{}%", (self.preview.zoom * 100.0).round() as i32),
                font(12.0, Weight::SemiBold),
                if resp.hovered() { TEXT_WHITE } else { TEXT_PRIMARY },
            );
            if resp.clicked() {
                self.preview.auto_fit = false;
                self.preview.zoom = 1.0;
            }
            if zoom_btn(ui, &self.icons, "minus", "").on_hover_text("Zoom out").clicked() {
                self.preview.auto_fit = false;
                self.preview.zoom = (self.preview.zoom - 0.1).clamp(ZOOM_MIN, ZOOM_MAX);
            }
        });

    }
}
