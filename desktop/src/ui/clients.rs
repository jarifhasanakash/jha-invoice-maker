//! Client management page.

use eframe::egui::{self, Frame, Margin, Stroke, Ui, Vec2, pos2};

use super::theme::*;
use super::widgets::*;
use super::{Action, App, ClientForm, CurrentClient, centered_page, page_header};
use crate::currency::{CURRENCIES, symbol_for};
use crate::fonts::Weight;
use crate::model::Client;

fn valid_email(s: &str) -> bool {
    // Same rule as the original: something@something.something, no spaces.
    if s.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((local, domain)) = s.split_once('@') else { return false };
    if local.is_empty() || domain.contains('@') {
        return false;
    }
    match domain.rsplit_once('.') {
        Some((a, b)) => !a.is_empty() && !b.is_empty(),
        None => false,
    }
}

impl App {
    pub(super) fn edit_client(&mut self, id: &str) {
        let Some(c) = self.db.find_client(id).cloned() else { return };
        self.clients.form = Some(ClientForm {
            id: c.id.clone(),
            name: c.name.clone(),
            email: c.email.clone(),
            location: c.location.clone(),
            phone: c.phone.clone(),
            currency: c.currency_code(),
            notes: c.notes.clone(),
            invalid: None,
        });
    }

    pub(super) fn delete_client(&mut self, frame: &eframe::Frame, id: &str) {
        let name = self.db.find_client(id).map(|c| c.name.clone()).unwrap_or_else(|| "this client".into());
        if !self.confirm(frame, "Delete client", &format!("Are you sure you want to delete {name}?")) {
            return;
        }
        self.db.delete_client(id);
        self.persist();
        self.toast("Client deleted", ToastKind::Info);
    }

    pub(super) fn save_client_form(&mut self) {
        let Some(form) = self.clients.form.as_mut() else { return };
        let name = form.name.trim().to_string();
        let email = form.email.trim().to_string();
        let location = form.location.trim().to_string();
        let error = if name.is_empty() {
            Some(("name", "Client / Company Name is required."))
        } else if email.is_empty() {
            Some(("email", "Email Address is required."))
        } else if !valid_email(&email) {
            Some(("email", "Please enter a valid email address."))
        } else if location.is_empty() {
            Some(("location", "Location (City, Country) is required."))
        } else {
            None
        };
        if let Some((field, msg)) = error {
            form.invalid = Some(field);
            self.toast(msg, ToastKind::Error);
            return;
        }
        let currency = if form.currency.is_empty() { "USD".to_string() } else { form.currency.clone() };
        let existing_extra = self
            .db
            .clients
            .iter()
            .find(|c| c.id == form.id)
            .map(|c| c.extra.clone())
            .unwrap_or_default();
        let client = Client {
            id: form.id.clone(),
            name: name.clone(),
            email,
            location,
            phone: form.phone.trim().to_string(),
            notes: form.notes.trim().to_string(),
            currency_symbol: Some(symbol_for(&currency).unwrap_or("$").to_string()),
            currency: Some(currency),
            extra: existing_extra,
            ..Default::default()
        };
        let saved = self.db.save_client(client);
        self.persist();
        self.clients.form = None;

        // Keep the editor in sync if this client is in use.
        let current = &self.ed.client;
        if current.id.as_deref() == Some(saved.id.as_str()) || current.name == saved.name {
            self.ed.client = CurrentClient {
                id: Some(saved.id.clone()),
                name: saved.name.clone(),
                location: saved.location.clone(),
                email: saved.email.clone(),
                currency: saved.currency_code(),
                symbol: saved.symbol(),
            };
        }
        self.toast(format!("Client \"{name}\" saved successfully!"), ToastKind::Success);
    }

    pub(super) fn clients_page(&mut self, ui: &mut Ui) {
        let mut sa = egui::ScrollArea::vertical();
        if let Some(y) = self.review_scroll.take() {
            sa = sa.vertical_scroll_offset(y);
        }
        sa
            .id_salt("clients-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                centered_page(ui, 1060.0, |ui| self.clients_content(ui));
            });

        let existing: std::collections::HashSet<&str> = self.db.clients.iter().map(|c| c.id.as_str()).collect();
        self.clients.selected.retain(|id| existing.contains(id.as_str()));
        if !self.clients.selected.is_empty() {
            let q = self.clients.search.trim().to_lowercase();
            let visible: Vec<String> = self
                .db
                .clients
                .iter()
                .filter(|c| {
                    q.is_empty()
                        || c.name.to_lowercase().contains(&q)
                        || c.email.to_lowercase().contains(&q)
                        || c.location.to_lowercase().contains(&q)
                })
                .map(|c| c.id.clone())
                .collect();
            self.clients_bulk_bar(ui, &visible);
        }
    }

    fn clients_content(&mut self, ui: &mut Ui) {
        let count = self.db.clients.len();
        page_header(
            ui,
            &self.icons,
            "users",
            "Manage Clients",
            "Save clients for instant one-click selection",
            |ui| {
                pill(
                    ui,
                    &format!("{count} {}", if count == 1 { "Client" } else { "Clients" }),
                    TEXT_SECONDARY,
                    BG_SURFACE_HOVER,
                    Some(BORDER_MUTED),
                    11.5,
                );
            },
        );

        // Toolbar: search + add button
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            let add_w = 170.0;
            let w = ui.available_width() - add_w - 12.0;
            ui.allocate_ui(egui::vec2(w, 42.0), |ui| {
                ui.set_width(w);
                let r = input(ui, &mut self.clients.search, "Search clients by name, email, location...", false);
                let ir = egui::Rect::from_center_size(pos2(r.rect.right() - 20.0, r.rect.center().y), Vec2::splat(15.0));
                self.icons.paint(ui.painter(), "magnifying-glass", ir, TEXT_SUBTLE);
            });
            if Btn::new(Kind::Primary, Some("user-plus"), "Add New Client")
                .width(add_w)
                .height(40.0)
                .show(ui, &self.icons)
                .clicked()
            {
                self.clients.form = match self.clients.form {
                    Some(ref f) if f.id.is_empty() => None,
                    _ => Some(ClientForm {
                        currency: "USD".into(),
                        ..Default::default()
                    }),
                };
            }
        });
        ui.add_space(18.0);

        if self.clients.form.is_some() {
            self.client_form(ui);
            ui.add_space(18.0);
        }

        // List
        let q = self.clients.search.trim().to_lowercase();
        let list: Vec<Client> = self
            .db
            .clients
            .iter()
            .filter(|c| {
                q.is_empty()
                    || c.name.to_lowercase().contains(&q)
                    || c.email.to_lowercase().contains(&q)
                    || c.location.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();
        if list.is_empty() {
            card(egui::Color32::TRANSPARENT, BORDER_SUBTLE, 9, Margin::same(40)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    self.icons.show(ui, "users", 34.0, TEXT_SUBTLE);
                    ui.add_space(6.0);
                    text(
                        ui,
                        if q.is_empty() {
                            "No clients saved yet. Click \"Add New Client\" above."
                        } else {
                            "No clients match your search."
                        },
                        13.0,
                        Weight::Regular,
                        TEXT_DIM,
                    );
                });
            });
            return;
        }
        ui.spacing_mut().item_spacing.y = 8.0;
        for c in &list {
            self.client_card(ui, c);
        }
    }

    fn client_form(&mut self, ui: &mut Ui) {
        let icons = &self.icons;
        let mut save = false;
        let mut cancel = false;
        let Some(form) = self.clients.form.as_mut() else { return };
        let editing = !form.id.is_empty();
        Frame::new()
            .fill(BG_SURFACE_ALT)
            .stroke(Stroke::new(1.0, BORDER_MUTED))
            .corner_radius(8)
            .inner_margin(Margin::same(18))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 14.0;
                ui.horizontal(|ui| {
                    icons.show(ui, if editing { "pencil-simple" } else { "user-plus" }, 16.0, TEXT_PRIMARY);
                    text(ui, if editing { "Edit Client" } else { "Add New Client" }, 14.0, Weight::SemiBold, TEXT_PRIMARY);
                });
                let invalid = form.invalid;
                let r = labeled_input(ui, "Client / Company Name", true, &mut form.name, "Acme Corporation", invalid == Some("name"));
                if r.changed() && invalid == Some("name") {
                    form.invalid = None;
                }
                two_columns(ui, |a, b| {
                    let r = labeled_input(a, "Email Address", true, &mut form.email, "billing@acme.com", invalid == Some("email"));
                    if r.changed() && invalid == Some("email") {
                        form.invalid = None;
                    }
                    let r = labeled_input(b, "Location (City, Country)", true, &mut form.location, "New York, USA", invalid == Some("location"));
                    if r.changed() && invalid == Some("location") {
                        form.invalid = None;
                    }
                });
                two_columns(ui, |a, b| {
                    labeled_input(a, "Phone", false, &mut form.phone, "+1 555-0199", false);
                    b.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 6.0;
                        field_label(ui, "Currency", false);
                        let options: Vec<String> = CURRENCIES.iter().map(|c| c.label.to_string()).collect();
                        let sel = CURRENCIES.iter().position(|c| c.code == form.currency);
                        let w = ui.available_width();
                        if let Some(i) = select(ui, icons, "client-currency", &options, sel, "USD - US Dollar ($)", w, 42.0, None, false) {
                            form.currency = CURRENCIES[i].code.to_string();
                        }
                    });
                });
                labeled_input(ui, "Note", false, &mut form.notes, "e.g. Net 14 days, VAT exempt", false);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    if Btn::new(Kind::Primary, Some("floppy-disk"), "Save Client").show(ui, icons).clicked() {
                        save = true;
                    }
                    if Btn::new(Kind::Tertiary, Some("x"), "Cancel").show(ui, icons).clicked() {
                        cancel = true;
                    }
                });
            });
        if save {
            self.actions.push(Action::SaveClientForm);
        }
        if cancel {
            self.clients.form = None;
        }
    }

    fn client_card(&mut self, ui: &mut Ui, c: &Client) {
        let selected = self.clients.selected.contains(&c.id);
        let id = ui.scope_id().with(("client-card", &c.id));
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
                // Everything on one line: [x] Name [CUR]  email  location ... [menu]
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    if checkbox(ui, &self.icons, if selected { 1 } else { 0 }).clicked() {
                        if selected {
                            self.clients.selected.remove(&c.id);
                        } else {
                            self.clients.selected.insert(c.id.clone());
                        }
                    }
                    let name_w = (ui.available_width() * 0.3).max(120.0);
                    let mut job = egui::text::LayoutJob::simple_singleline(
                        c.name.clone(),
                        font(14.5, Weight::SemiBold),
                        TEXT_WHITE,
                    );
                    job.wrap.max_width = name_w;
                    job.wrap.max_rows = 1;
                    ui.label(job).on_hover_text(&c.name);
                    pill(ui, &c.currency_code(), TEXT_MUTED, alpha(TEXT_WHITE, 0.06), Some(alpha(TEXT_WHITE, 0.1)), 10.0);
                    for (icon, value) in [("envelope-simple", &c.email), ("map-pin", &c.location)] {
                        if value.is_empty() {
                            continue;
                        }
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            self.icons.show(ui, icon, 14.0, TEXT_DIM);
                            let mut job = egui::text::LayoutJob::simple_singleline(
                                value.clone(),
                                font(12.5, Weight::Regular),
                                TEXT_DIM,
                            );
                            job.wrap.max_width = 240.0;
                            job.wrap.max_rows = 1;
                            ui.label(job).on_hover_text(value.as_str());
                        });
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let dots = dots_button(ui, false).on_hover_text("Client actions");
                        let items = [
                            ("arrow-square-in", "Use in Invoice", false, false),
                            ("pencil-simple", "Edit Client", false, false),
                            ("trash", "Delete", true, true),
                        ];
                        match row_menu(ui, &self.icons, &dots, &items) {
                            Some(0) => self.actions.push(Action::UseClient(c.id.clone())),
                            Some(1) => self.actions.push(Action::EditClient(c.id.clone())),
                            Some(2) => self.actions.push(Action::DeleteClient(c.id.clone())),
                            _ => {}
                        }
                    });
                });
            })
            .response;
        let hovered = ui.rect_contains_pointer(resp.rect);
        ui.ctx().data_mut(|d| d.insert_temp(id, hovered));
    }

    pub(super) fn bulk_delete_clients(&mut self, frame: &eframe::Frame) {
        let count = self.clients.selected.len();
        if count == 0 {
            return;
        }
        if !self.confirm(
            frame,
            "Delete clients",
            &format!("Are you sure you want to delete {count} selected client(s)? This action cannot be undone."),
        ) {
            return;
        }
        let ids: Vec<String> = self.clients.selected.drain().collect();
        for id in &ids {
            self.db.delete_client(id);
        }
        self.persist();
        self.toast(format!("Deleted {count} client(s) successfully!"), ToastKind::Success);
    }

    /// Floating bar shown while clients are selected (same as the history page).
    fn clients_bulk_bar(&mut self, ui: &mut Ui, visible_ids: &[String]) {
        let ctx = ui.ctx().clone();
        let on_page = visible_ids.iter().filter(|id| self.clients.selected.contains(*id)).count();
        let state = if on_page == 0 { 0 } else if on_page == visible_ids.len() { 1 } else { 2 };
        let count = self.clients.selected.len();
        egui::Area::new(egui::Id::unique("clients-bulk-bar"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -28.0))
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
                            if checkbox(ui, &self.icons, state).on_hover_text("Select / deselect all").clicked() {
                                if state == 1 {
                                    for id in visible_ids {
                                        self.clients.selected.remove(id);
                                    }
                                } else {
                                    self.clients.selected.extend(visible_ids.iter().cloned());
                                }
                            }
                            text(ui, &format!("{count} selected"), 13.0, Weight::SemiBold, TEXT_WHITE);
                            ui.add_space(6.0);
                            if Btn::new(Kind::Danger, Some("trash"), "Delete")
                                .height(34.0)
                                .show(ui, &self.icons)
                                .on_hover_text("Delete the selected clients")
                                .clicked()
                            {
                                self.actions.push(Action::BulkDeleteClients);
                            }
                            let (r, _) = ui.allocate_exact_size(egui::vec2(1.0, 22.0), egui::Sense::hover());
                            ui.painter().rect_filled(r, 0, BORDER_DEFAULT);
                            if icon_button(ui, &self.icons, "x", Vec2::splat(30.0), 13.0, TEXT_SECONDARY, alpha(TEXT_WHITE, 0.08), None)
                                .on_hover_text("Clear selection")
                                .clicked()
                            {
                                self.clients.selected.clear();
                            }
                        });
                    });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::valid_email;

    #[test]
    fn email_rule() {
        assert!(valid_email("a@b.co"));
        assert!(!valid_email("a@b"));
        assert!(!valid_email("a b@c.d"));
        assert!(!valid_email("@b.co"));
    }
}
