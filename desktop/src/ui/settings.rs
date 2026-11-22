//! Settings & database page.

use eframe::egui::{self, Margin, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, pos2};

use super::theme::*;
use super::widgets::*;
use super::{Action, App, SettingsForm, centered_page, page_header};
use crate::currency::{CURRENCIES, symbol_for};
use crate::fonts::Weight;
use crate::logo::{self, Logo};
use crate::model::Platform;

impl App {
    pub(super) fn load_settings_form(&mut self) {
        let p = &self.db.profile;
        let p1 = p.platform(0);
        let p2 = p.platform(1);
        let show_password = self.settings.show_password;
        self.settings = SettingsForm {
            first_name: p.first_name.clone(),
            last_name: p.last_name.clone(),
            email: p.email.clone(),
            mobile: p.mobile.clone(),
            address: p.address.clone(),
            currency: self.db.settings.currency_code(),
            p1_name: p1.name,
            p1_email: p1.email,
            p2_name: p2.name,
            p2_email: p2.email,
            bank_name: p.bank.bank_name.clone(),
            account_number: p.bank.account_number.clone(),
            branch_name: p.bank.branch_name.clone(),
            branch_code: p.bank.branch_code.clone(),
            swift_code: p.bank.swift_code.clone(),
            routing_no: p.bank.routing_no.clone(),
            pdf_password: p.pdf_password.clone(),
            show_password,
        };
    }

    pub(super) fn save_settings_form(&mut self) {
        let f = self.settings.clone();
        let p = &mut self.db.profile;
        p.first_name = f.first_name.trim().into();
        p.last_name = f.last_name.trim().into();
        p.email = f.email.trim().into();
        p.mobile = f.mobile.trim().into();
        p.address = f.address.trim().into();
        let mut platforms = std::mem::take(&mut p.platforms);
        platforms.resize_with(2.max(platforms.len()), Platform::default);
        let set = |pl: &mut Platform, id: &str, name: &str, default: &str, email: &str| {
            pl.id = id.into();
            pl.name = if name.trim().is_empty() { default.into() } else { name.trim().into() };
            pl.email = email.trim().into();
        };
        set(&mut platforms[0], "platform1", &f.p1_name, "Payoneer", &f.p1_email);
        set(&mut platforms[1], "platform2", &f.p2_name, "Wise", &f.p2_email);
        p.platforms = platforms;
        p.bank.bank_name = f.bank_name.trim().into();
        p.bank.account_number = f.account_number.trim().into();
        p.bank.branch_name = f.branch_name.trim().into();
        p.bank.branch_code = f.branch_code.trim().into();
        p.bank.swift_code = f.swift_code.trim().into();
        p.bank.routing_no = f.routing_no.trim().into();
        p.pdf_password = f.pdf_password.clone();
        self.db.settings.currency = Some(f.currency.clone());
        self.db.settings.currency_symbol = symbol_for(&f.currency).unwrap_or("$").to_string();
        if self.ed.client.id.is_none() {
            self.ed.client.currency = f.currency.clone();
            self.ed.client.symbol = self.db.settings.currency_symbol.clone();
        }
        self.persist();
        self.load_settings_form();
        self.toast("Settings, profile & PDF password saved successfully!", ToastKind::Success);
    }

    pub(super) fn upload_logo(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Upload Company Logo")
            .add_filter("Images", &["png", "jpg", "jpeg", "svg", "webp", "gif", "bmp"])
            .set_parent(frame)
            .pick_file()
        else {
            return;
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                self.toast(format!("Could not read the image: {e}"), ToastKind::Error);
                return;
            }
        };
        if let Err(e) = Logo::from_bytes(bytes.clone()) {
            self.toast(format!("This file can't be used as a logo: {e}"), ToastKind::Error);
            return;
        }
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        self.db.profile.company_logo = Some(logo::to_data_url(&bytes, &name));
        self.persist();
        self.refresh_logo(ctx);
        self.toast("Logo updated successfully!", ToastKind::Success);
    }

    pub(super) fn settings_page(&mut self, ui: &mut Ui) {
        let mut sa = egui::ScrollArea::vertical();
        if let Some(y) = self.review_scroll.take() {
            sa = sa.vertical_scroll_offset(y);
        }
        sa
            .id_salt("settings-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                centered_page(ui, 1060.0, |ui| self.settings_content(ui));
            });
    }

    fn settings_section(&mut self, ui: &mut Ui, icon: &str, title: &str, f: impl FnOnce(&mut Self, &mut Ui)) {
        section_heading(ui, Some((&self.icons, icon)), title);
        ui.add_space(4.0);
        ui.spacing_mut().item_spacing.y = 16.0;
        f(self, ui);
        ui.add_space(22.0);
    }

    fn settings_content(&mut self, ui: &mut Ui) {
        page_header(
            ui,
            &self.icons,
            "gear",
            "App Settings & Database",
            "Configure issuer details, payment methods, and JSON database backup",
            |_| {},
        );

        // ---- Logo + PDF protection cards ----
        let mut upload = false;
        let mut remove = false;
        let mut pwd_changed = false;
        {
            let icons = &self.icons;
            let thumb = self.logo_thumb.as_ref().map(|t| t.id());
            let has_logo = self.db.profile.company_logo.is_some();
            let form = &mut self.settings;
            two_columns(ui, |a, b| {
                let h = 190.0;
                card(egui::Color32::TRANSPARENT, BORDER_SUBTLE, 10, Margin::symmetric(22, 20)).show(a, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(h);
                    ui.spacing_mut().item_spacing.y = 10.0;
                    ui.horizontal(|ui| {
                        icons.show(ui, "image", 16.0, TEXT_PRIMARY);
                        text(ui, "Company Logo", 14.0, Weight::SemiBold, TEXT_PRIMARY);
                    });
                    wrapped(
                        ui,
                        "Upload your brand logo (PNG, JPG, or SVG) to display in the invoice header.",
                        12.5,
                        Weight::Regular,
                        TEXT_DIM,
                    );
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 16.0;
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(84.0), Sense::hover());
                        ui.painter().rect(r, 8, BG_SURFACE, Stroke::new(1.0, BORDER_SUBTLE), StrokeKind::Inside);
                        match thumb {
                            Some(id) => {
                                ui.painter().image(
                                    id,
                                    r.shrink(8.0),
                                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                    egui::Color32::WHITE,
                                );
                            }
                            None => {
                                ui.painter().text(
                                    r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "No Logo",
                                    font(11.5, Weight::Regular),
                                    TEXT_SUBTLE,
                                );
                            }
                        }
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 8.0;
                            let w = ui.available_width().min(220.0);
                            if Btn::new(Kind::Secondary, Some("upload-simple"), "Upload Logo").width(w).show(ui, icons).clicked() {
                                upload = true;
                            }
                            if Btn::new(Kind::Danger, Some("trash"), "Remove")
                                .width(w)
                                .enabled(has_logo)
                                .show(ui, icons)
                                .clicked()
                            {
                                remove = true;
                            }
                        });
                    });
                });
                card(egui::Color32::TRANSPARENT, BORDER_SUBTLE, 10, Margin::symmetric(22, 20)).show(b, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(h);
                    ui.spacing_mut().item_spacing.y = 10.0;
                    ui.horizontal(|ui| {
                        icons.show(ui, "shield-check", 16.0, TEXT_PRIMARY);
                        text(ui, "PDF Edit Protection & Security", 14.0, Weight::SemiBold, TEXT_PRIMARY);
                    });
                    wrapped(
                        ui,
                        "Protect exported invoice PDFs with strong AES-256 permissions security. Anyone can view, print, and copy freely without a password, but modifying or editing requires this password. Leave blank for unrestricted editing.",
                        12.5,
                        Weight::Regular,
                        TEXT_DIM,
                    );
                    ui.add_space(6.0);
                    ui.spacing_mut().item_spacing.y = 6.0;
                    field_label(ui, "PDF Edit Protection Password", false);
                    let r = password_input(
                        ui,
                        &mut form.pdf_password,
                        "Enter password to lock PDF editing (leave blank to allow all)",
                        form.show_password,
                    );
                    if r.changed() {
                        pwd_changed = true;
                    }
                    let br = Rect::from_center_size(pos2(r.rect.right() - 22.0, r.rect.center().y), Vec2::splat(30.0));
                    let eye = ui
                        .interact(br, ui.scope_id().with("eye"), Sense::click())
                        .on_hover_text("Show or hide password")
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if eye.hovered() {
                        ui.painter().rect_filled(br, 6, alpha(TEXT_WHITE, 0.08));
                    }
                    icons.paint(
                        ui.painter(),
                        if form.show_password { "eye-slash" } else { "eye" },
                        Rect::from_center_size(br.center(), Vec2::splat(16.0)),
                        if eye.hovered() { TEXT_WHITE } else { TEXT_DIM },
                    );
                    if eye.clicked() {
                        form.show_password = !form.show_password;
                    }
                });
            });
        }
        if upload {
            self.actions.push(Action::UploadLogo);
        }
        if remove {
            self.actions.push(Action::RemoveLogo);
        }
        if pwd_changed {
            // Saved as you type, like the original, so exports never miss it.
            self.actions.push(Action::SavePdfPassword);
        }
        ui.add_space(30.0);

        // ---- Profile ----
        self.settings_section(ui, "user", "Profile & \"From\" Details", |app, ui| {
            text(ui, "Default issuer information displayed in the \"From\" section of the invoice.", 13.0, Weight::Regular, TEXT_DIM);
            let f = &mut app.settings;
            two_columns(ui, |a, b| {
                labeled_input(a, "First Name", false, &mut f.first_name, "Jarif", false);
                labeled_input(b, "Last Name", false, &mut f.last_name, "Hasan", false);
            });
            two_columns(ui, |a, b| {
                labeled_input(a, "Email", false, &mut f.email, "jarif.hasan@example.com", false);
                labeled_input(b, "Phone", false, &mut f.mobile, "+880 1700-000000", false);
            });
            labeled_input(ui, "Address", false, &mut f.address, "Dhaka, Bangladesh", false);
        });

        // ---- Currency ----
        self.settings_section(ui, "currency-dollar", "Currency & Regional", |app, ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                field_label(ui, "Default Currency", false);
                let options: Vec<String> = CURRENCIES.iter().map(|c| c.label.to_string()).collect();
                let sel = CURRENCIES.iter().position(|c| c.code == app.settings.currency);
                let w = ui.available_width();
                if let Some(i) = select(ui, &app.icons, "default-currency", &options, sel, "USD - US Dollar ($)", w, 42.0, None, false) {
                    app.settings.currency = CURRENCIES[i].code.to_string();
                }
            });
        });

        // ---- Platforms ----
        self.settings_section(ui, "credit-card", "Payment Platforms (Default)", |app, ui| {
            let f = &mut app.settings;
            two_columns(ui, |a, b| {
                labeled_input(a, "Platform #1 Name", false, &mut f.p1_name, "Payoneer", false);
                labeled_input(b, "Platform #1 Email / ID", false, &mut f.p1_email, "jarif.hasan@example.com", false);
            });
            two_columns(ui, |a, b| {
                labeled_input(a, "Platform #2 Name", false, &mut f.p2_name, "Wise", false);
                labeled_input(b, "Platform #2 Email / ID", false, &mut f.p2_email, "jarif.hasan@example.com", false);
            });
        });

        // ---- Bank ----
        self.settings_section(ui, "bank", "Bank Details (Default)", |app, ui| {
            let f = &mut app.settings;
            two_columns(ui, |a, b| {
                labeled_input(a, "Bank Name", false, &mut f.bank_name, "Dhaka Bank", false);
                labeled_input(b, "A/C No", false, &mut f.account_number, "1234567890", false);
            });
            two_columns(ui, |a, b| {
                labeled_input(a, "Branch Name", false, &mut f.branch_name, "Banani Branch", false);
                labeled_input(b, "Branch Code", false, &mut f.branch_code, "012", false);
            });
            two_columns(ui, |a, b| {
                labeled_input(a, "SWIFT Code", false, &mut f.swift_code, "DHAKBDDHXXX", false);
                labeled_input(b, "Routing No", false, &mut f.routing_no, "085260123", false);
            });
        });

        // ---- Save bar ----
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if Btn::new(Kind::Primary, Some("check-circle"), "Save Profile & Settings")
                .height(40.0)
                .show(ui, &self.icons)
                .clicked()
            {
                self.actions.push(Action::SaveSettings);
            }
        });
        ui.add_space(34.0);

        // ---- Database ----
        self.settings_section(ui, "database", "JSON Database Engine", |app, ui| {
            card(BG_SURFACE, BORDER_SUBTLE, 8, Margin::same(18)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 14.0;
                ui.horizontal(|ui| {
                    let path = app.storage.path.display().to_string();
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        text(ui, "Stored locally in a JSON file on this computer. Back up or move across devices anytime.", 13.0, Weight::Regular, TEXT_DIM);
                        text(ui, &path, 11.5, Weight::Regular, TEXT_SUBTLE).on_hover_text(&path);
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        pill(ui, "Active", SUCCESS, alpha(SUCCESS, 0.1), Some(alpha(SUCCESS, 0.3)), 11.0);
                    });
                });
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    text(ui, &app.db.clients.len().to_string(), 13.0, Weight::Bold, TEXT_WHITE);
                    text(ui, if app.db.clients.len() == 1 { "Client" } else { "Clients" }, 13.0, Weight::Regular, TEXT_SECONDARY);
                    text(ui, "•", 13.0, Weight::Regular, TEXT_SUBTLE);
                    text(ui, &app.db.invoices.len().to_string(), 13.0, Weight::Bold, TEXT_WHITE);
                    text(ui, if app.db.invoices.len() == 1 { "Invoice" } else { "Invoices" }, 13.0, Weight::Regular, TEXT_SECONDARY);
                    text(ui, "•", 13.0, Weight::Regular, TEXT_SUBTLE);
                    let updated = chrono::DateTime::parse_from_rfc3339(&app.db.last_updated)
                        .map(|d| format!("Updated {}", d.with_timezone(&chrono::Local).format("%I:%M %p")))
                        .unwrap_or_else(|_| "Synced".into());
                    text(ui, &updated, 13.0, Weight::Regular, TEXT_SECONDARY);
                });
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    let w = (ui.available_width() - 20.0 - 150.0) / 2.0;
                    if Btn::new(Kind::Primary, Some("download-simple"), "Export JSON Database").width(w).show(ui, &app.icons).clicked() {
                        app.actions.push(Action::ExportDatabase);
                    }
                    if Btn::new(Kind::Secondary, Some("upload-simple"), "Restore JSON Database").width(w).show(ui, &app.icons).clicked() {
                        app.actions.push(Action::RestoreDatabase);
                    }
                    if Btn::new(Kind::Tertiary, Some("folder"), "Open Folder")
                        .width(150.0)
                        .show(ui, &app.icons)
                        .on_hover_text("Open the folder that holds database.json")
                        .clicked()
                    {
                        app.actions.push(Action::OpenDataFolder);
                    }
                });
            });
        });

        // ---- About ----
        self.settings_section(ui, "info", "About Application", |app, ui| {
            card(BG_SURFACE, BORDER_SUBTLE, 8, Margin::same(20)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 16.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(48.0), Sense::hover());
                    ui.painter().rect(r, 10, BG_SURFACE_HOVER, Stroke::new(1.0, BORDER_MUTED), StrokeKind::Inside);
                    if let Some(icon) = &app.app_icon {
                        ui.painter().image(icon.id(), r.shrink(6.0), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
                    }
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        text(ui, "JHA Invoice Maker", 16.0, Weight::SemiBold, TEXT_WHITE);
                        text(ui, "Professional Offline-First Invoicing System", 12.5, Weight::Regular, TEXT_DIM);
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        pill(ui, "Desktop Edition", BRAND, alpha(BRAND, 0.12), Some(alpha(BRAND, 0.35)), 11.0);
                    });
                });
                let info = [
                    ("CREATED BY", "Jarif Hasan Akash".to_string()),
                    ("CURRENT VERSION", format!("v{} (Native Rust)", env!("CARGO_PKG_VERSION"))),
                    ("ARCHITECTURE", "Native Rust app (egui + krilla)".to_string()),
                    ("STORAGE ENGINE", "Local JSON file (zero cloud tracking)".to_string()),
                ];
                for pair in info.chunks(2) {
                    two_columns(ui, |a, b| {
                        for (col, (k, v)) in [a, b].into_iter().zip(pair.iter()) {
                            col.spacing_mut().item_spacing.y = 3.0;
                            text(col, k, 10.5, Weight::SemiBold, TEXT_SUBTLE);
                            text(col, v, 13.0, Weight::Medium, TEXT_SECONDARY);
                        }
                    });
                }
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;
                    text(ui, "SECURITY", 10.5, Weight::SemiBold, TEXT_SUBTLE);
                    text(ui, "AES-256 vector PDF protection & edit lock", 13.0, Weight::Medium, TEXT_SECONDARY);
                });
            });
        });

        // ---- Danger zone ----
        ui.add_space(4.0);
        hline(ui, BORDER_SUBTLE);
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.horizontal(|ui| {
                    self.icons.show(ui, "warning", 15.0, DANGER_TEXT);
                    let mut job = egui::text::LayoutJob::default();
                    job.append(
                        "DANGER ZONE",
                        0.0,
                        egui::TextFormat {
                            font_id: font(13.0, Weight::SemiBold),
                            color: DANGER_TEXT,
                            extra_letter_spacing: 0.7,
                            ..Default::default()
                        },
                    );
                    ui.label(job);
                });
                text(
                    ui,
                    "Resetting will clear all your saved settings, clients, invoices and database records.",
                    13.0,
                    Weight::Regular,
                    TEXT_DIM,
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if Btn::new(Kind::Danger, Some("warning-circle"), "Delete all data").show(ui, &self.icons).clicked() {
                    self.actions.push(Action::ResetAll);
                }
            });
        });
    }
}
