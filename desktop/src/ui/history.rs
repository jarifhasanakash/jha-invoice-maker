//! Invoice history & earnings page.

use eframe::egui::{self, Align2, CursorIcon, Frame, Margin, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

use super::theme::*;
use super::widgets::*;
use super::{Action, App, centered_page, page_header};
use crate::dates;
use crate::fonts::Weight;
use crate::layout::fixed2;
use crate::model::{self, Invoice, STATUSES};

const PER_PAGE: usize = 20;

fn pagination_pages(current: usize, total: usize) -> Vec<Option<usize>> {
    if total <= 7 {
        return (1..=total).map(Some).collect();
    }
    if current <= 4 {
        return vec![Some(1), Some(2), Some(3), Some(4), Some(5), None, Some(total)];
    }
    if current >= total - 3 {
        return vec![Some(1), None, Some(total - 4), Some(total - 3), Some(total - 2), Some(total - 1), Some(total)];
    }
    vec![Some(1), None, Some(current - 1), Some(current), Some(current + 1), None, Some(total)]
}

impl App {
    /// Invoices matching the history filters, newest first.
    pub(super) fn filtered_invoices(&self) -> Vec<&Invoice> {
        let h = &self.hist;
        let q = h.search.trim().to_lowercase();
        let status = h.status.map(|i| STATUSES[i].to_lowercase());
        let mut list: Vec<&Invoice> = self
            .db
            .invoices
            .iter()
            .filter(|i| {
                status
                    .as_ref()
                    .is_none_or(|s| i.status_or_default().to_lowercase() == *s)
            })
            .filter(|i| {
                let d = dates::parse(&i.date);
                h.from.is_none_or(|f| d.is_some_and(|d| d >= f))
                    && h.to.is_none_or(|t| d.is_some_and(|d| d <= t))
            })
            .filter(|i| {
                q.is_empty()
                    || i.invoice_number.to_lowercase().contains(&q)
                    || i.client.name.to_lowercase().contains(&q)
            })
            .collect();
        model::sort_invoices_desc(&mut list);
        list
    }

    pub(super) fn history_page(&mut self, ui: &mut Ui) {
        // Drop selections of invoices that no longer exist.
        let existing: std::collections::HashSet<&str> = self.db.invoices.iter().map(|i| i.id.as_str()).collect();
        self.hist.selected.retain(|id| existing.contains(id.as_str()));

        let mut sa = egui::ScrollArea::vertical();
        if let Some(y) = self.review_scroll.take() {
            sa = sa.vertical_scroll_offset(y);
        }
        sa
            .id_salt("history-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                centered_page(ui, 1060.0, |ui| self.history_content(ui));
            });

        if !self.hist.selected.is_empty() {
            self.bulk_bar(ui);
        }
    }

    fn history_content(&mut self, ui: &mut Ui) {
        page_header(
            ui,
            &self.icons,
            "chart-bar",
            "Invoice History & Earnings",
            "Track, filter, export and manage all created invoices",
            |_| {},
        );

        let filtered: Vec<Invoice> = self.filtered_invoices().into_iter().cloned().collect();

        // ---- Stats ----
        let (mut invoiced, mut paid, mut pending) = (0.0, 0.0, 0.0);
        for i in &filtered {
            invoiced += i.total;
            match i.status_or_default().to_lowercase().as_str() {
                "paid" => paid += i.total,
                "pending" | "overdue" => pending += i.total,
                _ => {}
            }
        }
        let symbols: std::collections::HashSet<String> = filtered.iter().map(|i| i.symbol()).collect();
        let sym = if symbols.len() == 1 {
            symbols.into_iter().next().unwrap_or_default()
        } else {
            self.settings_symbol()
        };
        ui.spacing_mut().item_spacing.x = 12.0;
        ui.columns(4, |cols| {
            let stats = [
                ("TOTAL INVOICED", format!("{sym}{}", fixed2(invoiced)), TEXT_WHITE),
                ("TOTAL PAID", format!("{sym}{}", fixed2(paid)), SUCCESS),
                ("PENDING / UNPAID", format!("{sym}{}", fixed2(pending)), WARNING),
                ("INVOICES COUNT", filtered.len().to_string(), TEXT_WHITE),
            ];
            for (col, (label, value, color)) in cols.iter_mut().zip(stats) {
                card(BG_SURFACE, BORDER_SUBTLE, 8, Margin::symmetric(16, 14)).show(col, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = 6.0;
                    let mut job = egui::text::LayoutJob::default();
                    job.append(
                        label,
                        0.0,
                        egui::TextFormat {
                            font_id: font(11.0, Weight::SemiBold),
                            color: TEXT_DIM,
                            extra_letter_spacing: 0.5,
                            ..Default::default()
                        },
                    );
                    ui.label(job);
                    text(ui, &value, 20.0, Weight::Bold, color);
                });
            }
        });
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.add_space(22.0);

        // ---- Filters ----
        let mut reset_page = false;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            let total_w = ui.available_width();
            let small = (total_w - 36.0) * 0.2;
            let search_w = total_w - 36.0 - small * 3.0;

            ui.vertical(|ui| {
                ui.set_width(search_w);
                ui.spacing_mut().item_spacing.y = 6.0;
                field_label(ui, "Search Invoices", false);
                let r = input(ui, &mut self.hist.search, "Search by invoice #, client...", false);
                let ir = egui::Rect::from_center_size(pos2(r.rect.right() - 20.0, r.rect.center().y), Vec2::splat(15.0));
                self.icons.paint(ui.painter(), "magnifying-glass", ir, TEXT_SUBTLE);
                if r.changed() {
                    reset_page = true;
                }
            });
            ui.vertical(|ui| {
                ui.set_width(small);
                ui.spacing_mut().item_spacing.y = 6.0;
                field_label(ui, "Status", false);
                let mut options = vec!["All Statuses".to_string()];
                options.extend(STATUSES.iter().map(|s| s.to_string()));
                let sel = Some(self.hist.status.map(|i| i + 1).unwrap_or(0));
                if let Some(i) = select(ui, &self.icons, "hist-status", &options, sel, "All Statuses", small, 42.0, None, false) {
                    self.hist.status = if i == 0 { None } else { Some(i - 1) };
                    reset_page = true;
                }
            });
            ui.vertical(|ui| {
                ui.set_width(small);
                ui.spacing_mut().item_spacing.y = 6.0;
                field_label(ui, "From Date", false);
                match date_field(ui, &self.icons, "hist-from", self.hist.from, "Clear") {
                    Some(DateAction::Picked(d)) => {
                        self.hist.from = Some(d);
                        reset_page = true;
                    }
                    Some(DateAction::Cleared) => {
                        self.hist.from = None;
                        reset_page = true;
                    }
                    None => {}
                }
            });
            ui.vertical(|ui| {
                ui.set_width(small);
                ui.spacing_mut().item_spacing.y = 6.0;
                field_label(ui, "To Date", false);
                match date_field(ui, &self.icons, "hist-to", self.hist.to, "Clear") {
                    Some(DateAction::Picked(d)) => {
                        self.hist.to = Some(d);
                        reset_page = true;
                    }
                    Some(DateAction::Cleared) => {
                        self.hist.to = None;
                        reset_page = true;
                    }
                    None => {}
                }
            });
        });
        if reset_page {
            self.hist.page = 0;
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            if Btn::new(Kind::Secondary, Some("file-csv"), "Export Filtered CSV")
                .show(ui, &self.icons)
                .on_hover_text("Export the filtered records as CSV")
                .clicked()
            {
                self.actions.push(Action::ExportCsv);
            }
            if Btn::new(Kind::Secondary, Some("file-code"), "Export Filtered JSON")
                .show(ui, &self.icons)
                .on_hover_text("Export the filtered invoices as JSON")
                .clicked()
            {
                self.actions.push(Action::ExportInvoicesJson);
            }
        });
        ui.add_space(26.0);

        // ---- List header ----
        let total = filtered.len();
        let pages = total.div_ceil(PER_PAGE).max(1);
        if self.hist.page >= pages {
            self.hist.page = pages - 1;
        }
        let start = self.hist.page * PER_PAGE;
        let end = (start + PER_PAGE).min(total);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            text(ui, "All Invoices", 15.0, Weight::SemiBold, TEXT_WHITE);
            pill(
                ui,
                &format!("{total} {}", if total == 1 { "Invoice" } else { "Invoices" }),
                TEXT_DIM,
                BG_SURFACE_HOVER,
                None,
                11.0,
            );
            if total > 0 {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let noun = if total == 1 { "invoice" } else { "invoices" };
                    text(ui, noun, 12.5, Weight::Regular, TEXT_DIM);
                    text(ui, &total.to_string(), 12.5, Weight::SemiBold, TEXT_SECONDARY);
                    text(ui, "of", 12.5, Weight::Regular, TEXT_DIM);
                    text(ui, &format!("{}–{}", start + 1, end), 12.5, Weight::SemiBold, TEXT_SECONDARY);
                    text(ui, "Showing", 12.5, Weight::Regular, TEXT_DIM);
                });
            }
        });
        ui.add_space(12.0);

        // ---- Cards ----
        if total == 0 {
            let filtered_view = !self.hist.search.trim().is_empty()
                || self.hist.status.is_some()
                || self.hist.from.is_some()
                || self.hist.to.is_some();
            card(egui::Color32::TRANSPARENT, BORDER_SUBTLE, 9, Margin::same(40)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    self.icons.show(ui, "receipt", 36.0, TEXT_SUBTLE);
                    ui.add_space(6.0);
                    text(
                        ui,
                        if filtered_view {
                            "No invoices match your search filters."
                        } else {
                            "No invoices created yet. Click \"+ New\" above to create an invoice."
                        },
                        13.0,
                        Weight::Regular,
                        TEXT_DIM,
                    );
                });
            });
        } else {
            ui.spacing_mut().item_spacing.y = 8.0;
            for inv in &filtered[start..end] {
                self.invoice_card(ui, inv);
            }
        }

        // ---- Pagination ----
        if pages > 1 {
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let items = pagination_pages(self.hist.page + 1, pages);
                let est = 80.0 * 2.0 + items.len() as f32 * 40.0;
                ui.add_space(((ui.available_width() - est) / 2.0).max(0.0));
                let cur = self.hist.page + 1;
                if self.page_btn(ui, "Prev", Some("caret-left"), false, cur == 1).clicked() {
                    self.hist.page -= 1;
                }
                for p in items {
                    match p {
                        Some(p) => {
                            if self.page_btn(ui, &p.to_string(), None, p == cur, false).clicked() {
                                self.hist.page = p - 1;
                            }
                        }
                        None => {
                            text(ui, "…", 13.0, Weight::Regular, TEXT_DIM);
                        }
                    }
                }
                if self.page_btn(ui, "Next", Some("caret-right"), false, cur == pages).clicked() {
                    self.hist.page += 1;
                }
            });
        }
        ui.add_space(if self.hist.selected.is_empty() { 0.0 } else { 70.0 });
    }

    fn page_btn(&self, ui: &mut Ui, label: &str, icon: Option<&str>, active: bool, disabled: bool) -> egui::Response {
        let g = galley(ui, label, 13.0, if active { Weight::SemiBold } else { Weight::Regular }, TEXT_WHITE);
        let w = (g.size().x + 20.0 + if icon.is_some() { 18.0 } else { 0.0 }).max(34.0);
        let (r, resp) = ui.allocate_exact_size(vec2(w, 34.0), if disabled { Sense::hover() } else { Sense::click() });
        let (bg, border, fg) = if active {
            (BRAND, BRAND, TEXT_WHITE)
        } else if disabled {
            (BG_SURFACE, BORDER_SUBTLE, alpha(TEXT_SECONDARY, 0.35))
        } else if resp.hovered() {
            (BG_SURFACE_HOVER, BORDER_LIGHT, TEXT_WHITE)
        } else {
            (BG_SURFACE, BORDER_SUBTLE, TEXT_SECONDARY)
        };
        ui.painter().rect(r, 6, bg, Stroke::new(1.0, border), StrokeKind::Inside);
        let g = galley(ui, label, 13.0, if active { Weight::SemiBold } else { Weight::Regular }, fg);
        let total_w = g.size().x + if icon.is_some() { 18.0 } else { 0.0 };
        let mut x = r.center().x - total_w / 2.0;
        let icon_first = label == "Prev";
        if let (Some(icon), true) = (icon, icon_first) {
            self.icons.paint(ui.painter(), icon, egui::Rect::from_min_size(pos2(x, r.center().y - 7.0), Vec2::splat(14.0)), fg);
            x += 18.0;
        }
        let gw = g.size().x;
        ui.painter().galley(pos2(x, r.center().y - g.size().y / 2.0), g, fg);
        if let (Some(icon), false) = (icon, icon_first) {
            self.icons.paint(ui.painter(), icon, egui::Rect::from_min_size(pos2(x + gw + 4.0, r.center().y - 7.0), Vec2::splat(14.0)), fg);
        }
        if disabled { resp } else { resp.on_hover_cursor(CursorIcon::PointingHand) }
    }

    fn invoice_card(&mut self, ui: &mut Ui, inv: &Invoice) {
        let selected = self.hist.selected.contains(&inv.id);
        let id = ui.scope_id().with(("inv-card", &inv.id));
        let hovered_prev = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
        let (fill, border) = if selected {
            (alpha(BRAND, 0.06), alpha(BRAND, 0.45))
        } else if hovered_prev {
            (BG_SURFACE_ALT, BORDER_DEFAULT)
        } else {
            (BG_APP, BORDER_SUBTLE)
        };
        let resp = Frame::new()
            .fill(fill)
            .stroke(Stroke::new(1.0, border))
            .corner_radius(9)
            .inner_margin(Margin::symmetric(18, 12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    if checkbox(ui, &self.icons, if selected { 1 } else { 0 }).clicked() {
                        if selected {
                            self.hist.selected.remove(&inv.id);
                        } else {
                            self.hist.selected.insert(inv.id.clone());
                        }
                    }
                    pill(
                        ui,
                        &format!("#{}", inv.invoice_number),
                        TEXT_SECONDARY,
                        BG_SURFACE,
                        Some(BORDER_SUBTLE),
                        13.0,
                    );
                    // Client name and date on a single line.
                    let name = inv.client_name();
                    let max_w = (ui.available_width() - 460.0).max(120.0);
                    let mut job = egui::text::LayoutJob::simple_singleline(name.clone(), font(14.5, Weight::SemiBold), TEXT_WHITE);
                    job.wrap.max_width = max_w;
                    job.wrap.max_rows = 1;
                    ui.label(job).on_hover_text(name);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        self.icons.show(ui, "calendar-blank", 13.0, TEXT_DIM);
                        text(ui, &dates::display_str(&inv.date), 12.5, Weight::Regular, TEXT_DIM);
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        let dots = dots_button(ui, false).on_hover_text("Invoice actions");
                        let items = [
                            ("pencil-simple", "Edit Invoice", false, false),
                            ("download-simple", "Download PDF", false, false),
                            ("copy", "Duplicate", false, false),
                            ("trash", "Delete", true, true),
                        ];
                        match row_menu(ui, &self.icons, &dots, &items) {
                            Some(0) => self.actions.push(Action::LoadInvoice(inv.id.clone())),
                            Some(1) => self.actions.push(Action::DownloadInvoice(inv.id.clone())),
                            Some(2) => self.actions.push(Action::DuplicateInvoice(inv.id.clone())),
                            Some(3) => self.actions.push(Action::DeleteInvoice(inv.id.clone())),
                            _ => {}
                        }

                        let status = inv.status_or_default().to_string();
                        let options: Vec<String> = STATUSES.iter().map(|s| s.to_string()).collect();
                        let sel = STATUSES.iter().position(|s| *s == status);
                        if let Some(i) = select(
                            ui,
                            &self.icons,
                            ("status", &inv.id),
                            &options,
                            sel,
                            &status,
                            112.0,
                            32.0,
                            Some(status_color(&status)),
                            false,
                        ) {
                            if STATUSES[i] != status {
                                self.actions.push(Action::SetStatus(inv.id.clone(), STATUSES[i].to_string()));
                            }
                        }

                        let amount = format!("{}{}", inv.symbol(), fixed2(inv.total));
                        let g = galley(ui, &amount, 14.5, Weight::SemiBold, TEXT_WHITE);
                        let (r, _) = ui.allocate_exact_size(vec2(g.size().x.max(90.0), 24.0), Sense::hover());
                        ui.painter().galley(
                            pos2(r.right() - g.size().x, r.center().y - g.size().y / 2.0),
                            g,
                            TEXT_WHITE,
                        );
                    });
                });
            })
            .response;
        let hovered = ui.rect_contains_pointer(resp.rect);
        ui.ctx().data_mut(|d| d.insert_temp(id, hovered));
    }

    fn bulk_bar(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let filtered_ids: Vec<String> = {
            let list = self.filtered_invoices();
            let start = self.hist.page * PER_PAGE;
            let end = (start + PER_PAGE).min(list.len());
            list[start.min(end)..end].iter().map(|i| i.id.clone()).collect()
        };
        let on_page = filtered_ids.iter().filter(|id| self.hist.selected.contains(*id)).count();
        let state = if on_page == 0 {
            0
        } else if on_page == filtered_ids.len() {
            1
        } else {
            2
        };
        let count = self.hist.selected.len();
        egui::Area::new(egui::Id::unique("bulk-bar"))
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -28.0))
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                Frame::new()
                    .fill(egui::Color32::from_rgb(0x1c, 0x1c, 0x1f))
                    .stroke(Stroke::new(1.0, alpha(BRAND, 0.5)))
                    .corner_radius(50)
                    .inner_margin(Margin { left: 18, right: 12, top: 8, bottom: 8 })
                    .shadow(egui::Shadow {
                        offset: [0, 12],
                        blur: 32,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(160),
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 14.0;
                            if checkbox(ui, &self.icons, state)
                                .on_hover_text("Select / deselect all on this page")
                                .clicked()
                            {
                                if state == 1 {
                                    for id in &filtered_ids {
                                        self.hist.selected.remove(id);
                                    }
                                } else {
                                    self.hist.selected.extend(filtered_ids.iter().cloned());
                                }
                            }
                            text(ui, &format!("{count} selected"), 13.0, Weight::SemiBold, TEXT_WHITE);
                            ui.add_space(6.0);
                            if Btn::new(Kind::Secondary, Some("download-simple"), "Download")
                                .height(34.0)
                                .show(ui, &self.icons)
                                .on_hover_text("Download the selected PDFs to a folder")
                                .clicked()
                            {
                                self.actions.push(Action::BulkDownload);
                            }
                            if Btn::new(Kind::Danger, Some("trash"), "Delete")
                                .height(34.0)
                                .show(ui, &self.icons)
                                .on_hover_text("Delete the selected invoices")
                                .clicked()
                            {
                                self.actions.push(Action::BulkDelete);
                            }
                            let (r, _) = ui.allocate_exact_size(vec2(1.0, 22.0), Sense::hover());
                            ui.painter().rect_filled(r, 0, BORDER_DEFAULT);
                            let resp = icon_button(
                                ui,
                                &self.icons,
                                "x",
                                Vec2::splat(30.0),
                                13.0,
                                TEXT_SECONDARY,
                                alpha(TEXT_WHITE, 0.08),
                                None,
                            )
                            .on_hover_text("Clear selection");
                            if resp.clicked() {
                                self.hist.selected.clear();
                            }
                        });
                    });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::pagination_pages;

    #[test]
    fn pagination_matches_original() {
        assert_eq!(pagination_pages(1, 3), vec![Some(1), Some(2), Some(3)]);
        assert_eq!(pagination_pages(2, 10), vec![Some(1), Some(2), Some(3), Some(4), Some(5), None, Some(10)]);
        assert_eq!(pagination_pages(9, 10), vec![Some(1), None, Some(6), Some(7), Some(8), Some(9), Some(10)]);
        assert_eq!(pagination_pages(5, 10), vec![Some(1), None, Some(4), Some(5), Some(6), None, Some(10)]);
    }
}
