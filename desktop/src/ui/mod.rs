//! Application shell: state, top bar, tab routing and actions.

mod clients;
mod editor;
mod history;
mod settings;
pub mod theme;
pub mod widgets;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::NaiveDate;
use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, Frame, Margin, Rect, Sense, Stroke, StrokeKind, Ui,
    Vec2, pos2, vec2,
};

use crate::dates;
use crate::fonts::Weight;
use crate::layout::{self, InvoiceView};
use crate::logo::Logo;
use crate::model::{self, Database, Invoice, InvoiceClient, LineItem, Party};
use crate::pdf;
use crate::storage::{LoadOutcome, Storage};
use theme::*;
use widgets::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Invoice,
    History,
    Clients,
    Settings,
}

/// The client currently used by the invoice editor.
#[derive(Clone, Debug, PartialEq)]
pub struct CurrentClient {
    pub id: Option<String>,
    pub name: String,
    pub location: String,
    pub email: String,
    pub currency: String,
    pub symbol: String,
}

impl CurrentClient {
    fn placeholder(currency: String, symbol: String) -> Self {
        Self {
            id: None,
            name: "Client Name".into(),
            location: "City/State, Country".into(),
            email: "client@example.com".into(),
            currency,
            symbol,
        }
    }
    fn is_placeholder(&self) -> bool {
        self.id.is_none() && (self.name.trim().is_empty() || self.name == "Client Name")
    }
}

pub struct EditorState {
    pub editing_id: Option<String>,
    pub status: String,
    pub number: String,
    pub date: NaiveDate,
    pub client: CurrentClient,
    pub items: Vec<LineItem>,
    pub price_text: Vec<String>,
    pub user_modified: bool,
    pub toggle_advance: bool,
    pub advance_text: String,
    pub toggle_balance: bool,
    pub note: String,
    /// Validation flash highlights (until time).
    pub flash_client: f64,
    pub flash_items: f64,
}

pub struct PreviewState {
    pub zoom: f32,
    pub auto_fit: bool,
    texture: Option<egui::TextureHandle>,
    rendered: Option<(InvoiceView, f32, usize)>,
    page_size: Vec2,
    rasterizer: crate::render::Rasterizer,
    /// Where the page was drawn last frame (for zoom-at-pointer).
    last_paper: Option<Rect>,
}

#[derive(Default)]
pub struct HistoryState {
    pub search: String,
    pub status: Option<usize>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub page: usize,
    pub selected: HashSet<String>,
}

#[derive(Default, Clone)]
pub struct ClientForm {
    pub id: String,
    pub name: String,
    pub email: String,
    pub location: String,
    pub phone: String,
    pub currency: String,
    pub notes: String,
    pub invalid: Option<&'static str>,
}

#[derive(Default)]
pub struct ClientsState {
    pub search: String,
    pub form: Option<ClientForm>,
    pub selected: HashSet<String>,
}

#[derive(Default, Clone)]
pub struct SettingsForm {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub mobile: String,
    pub address: String,
    pub currency: String,
    pub p1_name: String,
    pub p1_email: String,
    pub p2_name: String,
    pub p2_email: String,
    pub bank_name: String,
    pub account_number: String,
    pub branch_name: String,
    pub branch_code: String,
    pub swift_code: String,
    pub routing_no: String,
    pub pdf_password: String,
    pub show_password: bool,
}

/// Deferred actions, executed after the UI pass (so native dialogs can be
/// parented to the window and state can be mutated freely).
pub enum Action {
    SwitchTab(Tab),
    NewInvoice { silent: bool },
    SaveInvoice,
    DownloadCurrent,
    SelectClient(Option<String>),
    LoadInvoice(String),
    DuplicateInvoice(String),
    DeleteInvoice(String),
    DownloadInvoice(String),
    SetStatus(String, String),
    BulkDownload,
    BulkDelete,
    ExportCsv,
    ExportInvoicesJson,
    UseClient(String),
    EditClient(String),
    DeleteClient(String),
    BulkDeleteClients,
    SaveClientForm,
    UploadLogo,
    RemoveLogo,
    SaveSettings,
    SavePdfPassword,
    ExportDatabase,
    RestoreDatabase,
    ResetAll,
    OpenDataFolder,
}

pub struct App {
    storage: Storage,
    db: Database,
    tab: Tab,
    icons: Icons,
    toasts: Vec<Toast>,
    actions: Vec<Action>,
    ed: EditorState,
    preview: PreviewState,
    hist: HistoryState,
    clients: ClientsState,
    settings: SettingsForm,
    logo: Arc<Logo>,
    logo_src: Option<String>,
    logo_thumb: Option<egui::TextureHandle>,
    app_icon: Option<egui::TextureHandle>,
    /// Design-review: initial scroll offset for the first page shown (JHA_SCROLL).
    review_scroll: Option<f32>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let storage = Storage::locate();
        let (db, outcome) = storage.load();

        let currency = db.settings.currency_code();
        let symbol = db.settings.symbol();
        let mut app = Self {
            icons: Icons::load(&cc.egui_ctx),
            storage,
            db,
            tab: Tab::Invoice,
            toasts: Vec::new(),
            actions: Vec::new(),
            ed: EditorState {
                editing_id: None,
                status: "Pending".into(),
                number: "0001".into(),
                date: dates::today(),
                client: CurrentClient::placeholder(currency, symbol),
                items: Vec::new(),
                price_text: Vec::new(),
                user_modified: false,
                toggle_advance: false,
                advance_text: "0".into(),
                toggle_balance: false,
                note: model::DEFAULT_NOTE.into(),
                flash_client: 0.0,
                flash_items: 0.0,
            },
            preview: PreviewState {
                zoom: 1.0,
                auto_fit: true,
                texture: None,
                rendered: None,
                page_size: vec2(layout::PAPER_WIDTH, layout::PAPER_MIN_HEIGHT),
                rasterizer: Default::default(),
                last_paper: None,
            },
            hist: HistoryState::default(),
            clients: ClientsState::default(),
            settings: SettingsForm::default(),
            logo: Logo::default_brand(),
            logo_src: None,
            logo_thumb: None,
            app_icon: None,
            review_scroll: std::env::var("JHA_SCROLL").ok().and_then(|s| s.parse().ok()),
        };

        if let Ok(img) = image::load_from_memory(include_bytes!("../../assets/app-icon.png")) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let ci = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            app.app_icon = Some(cc.egui_ctx.load_texture("app-icon", ci, egui::TextureOptions::LINEAR));
        }

        app.load_settings_form();
        app.start_new_invoice(true);
        // Design-review: open on a given page (JHA_START_TAB=history|clients|settings).
        app.tab = match std::env::var("JHA_START_TAB").as_deref() {
            Ok("history") => Tab::History,
            Ok("clients") => Tab::Clients,
            Ok("settings") => Tab::Settings,
            _ => Tab::Invoice,
        };
        app.refresh_logo(&cc.egui_ctx);

        match outcome {
            LoadOutcome::Loaded => {}
            LoadOutcome::Fresh => {
                // First run: save the demo data so it's there next time too.
                app.persist();
                app.toast("Welcome! Sample data was added — edit it in Settings and Clients.", ToastKind::Info);
            }
            LoadOutcome::ImportedFromElectron { clients, invoices } => {
                app.persist();
                app.toast(
                    format!("Imported your data from the previous version: {clients} clients, {invoices} invoices."),
                    ToastKind::Success,
                );
            }
            LoadOutcome::Recovered { backup, error } => {
                app.toast(
                    format!(
                        "Your database could not be read ({error}). A copy was kept at {}.",
                        backup.display()
                    ),
                    ToastKind::Error,
                );
            }
        }
        app
    }

    /* ---------------- Helpers ---------------- */

    pub fn toast(&mut self, message: impl Into<String>, kind: ToastKind) {
        self.toasts.push(Toast {
            message: message.into(),
            kind,
            born: -1.0,
        });
    }

    fn persist(&mut self) {
        self.db.touch();
        if let Err(e) = self.storage.save(&self.db) {
            self.toast(format!("Could not save database: {e}"), ToastKind::Error);
        }
    }

    fn settings_symbol(&self) -> String {
        self.db.settings.symbol()
    }

    fn refresh_logo(&mut self, ctx: &egui::Context) {
        let src = self.db.profile.company_logo.clone();
        if src == self.logo_src && self.logo_thumb.is_some() {
            return;
        }
        self.logo_src = src.clone();
        self.logo = match src.as_deref().map(Logo::from_data_url) {
            Some(Ok(l)) => Arc::new(l),
            Some(Err(e)) => {
                self.toast(format!("Saved logo could not be loaded: {e}"), ToastKind::Warn);
                Logo::default_brand()
            }
            None => Logo::default_brand(),
        };
        // Thumbnail for the settings card (rendered through the page rasteriser).
        self.logo_thumb = None;
        if src.is_some() {
            let page = layout::PageLayout {
                width: 140.0,
                height: 140.0,
                background: [0x24, 0x24, 0x24],
                items: vec![layout::Paint::Logo { x: 0.0, y: 0.0, w: 140.0, h: 140.0 }],
                logo: self.logo.clone(),
            };
            let (px, size) = self.preview.rasterizer.render(&page, 2.0);
            let ci = egui::ColorImage::from_rgba_premultiplied(size, &px);
            self.logo_thumb = Some(ctx.load_texture("logo-thumb", ci, egui::TextureOptions::LINEAR));
        }
        self.preview.rendered = None;
    }

    fn confirm(&self, frame: &eframe::Frame, title: &str, message: &str) -> bool {
        let res = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Warning)
            .set_title(title)
            .set_description(message)
            .set_buttons(rfd::MessageButtons::OkCancel)
            .set_parent(frame)
            .show();
        matches!(res, rfd::MessageDialogResult::Ok | rfd::MessageDialogResult::Yes)
    }

    /* ---------------- Invoice view model ---------------- */

    /// Builds what's printed on the invoice from a stored/edited invoice and the current profile.
    fn view_for(&self, inv: &Invoice) -> InvoiceView {
        let p = &self.db.profile;
        let custom = p.has_custom_details();
        let full = p.full_name();
        let from_name = if full.is_empty() { "Jarif Hasan".to_string() } else { full.clone() };
        let (from_location, from_email) = if custom {
            (p.address.clone(), p.email.clone())
        } else {
            ("Dhaka, Bangladesh".to_string(), "jarif.hasan@example.com".to_string())
        };
        let mobile = if !p.mobile.is_empty() {
            p.mobile.clone()
        } else if custom {
            String::new()
        } else {
            "+880 1700-000000".into()
        };
        let p1 = p.platform(0);
        let p2 = p.platform(1);
        let b = &p.bank;
        InvoiceView {
            number: if inv.invoice_number.trim().is_empty() {
                "0001".into()
            } else {
                inv.invoice_number.trim().to_string()
            },
            date: dates::display_str(&inv.date),
            client_name: if inv.client.name.is_empty() { "Client Name".into() } else { inv.client.name.clone() },
            to_location: inv.client.location.clone(),
            to_email: inv.client.email.clone(),
            from_name: from_name.clone(),
            from_location,
            from_email,
            items: inv.items.clone(),
            symbol: inv.symbol(),
            show_advance: inv.toggle_advance,
            show_balance: inv.toggle_balance,
            advance: inv.advance_amount,
            note: inv.note.clone(),
            platforms: [(p1.name, p1.email), (p2.name, p2.email)],
            bank_left: vec![
                ("First Name".into(), p.first_name.clone()),
                ("Last Name".into(), p.last_name.clone()),
                ("Address".into(), p.address.clone()),
                ("A/C No".into(), b.account_number.clone()),
            ],
            bank_right: vec![
                ("Bank Name".into(), b.bank_name.clone()),
                ("Branch Name".into(), b.branch_name.clone()),
                ("Branch Code".into(), b.branch_code.clone()),
                ("SWIFT Code".into(), b.swift_code.clone()),
                ("Routing No".into(), b.routing_no.clone()),
            ],
            signature_name: from_name,
            mobile,
        }
    }

    /// The invoice record represented by the editor's current state.
    fn editor_invoice(&self) -> Invoice {
        let ed = &self.ed;
        let total: f64 = ed.items.iter().map(|i| i.price).sum();
        let advance = if ed.toggle_advance {
            model::parse_float_prefix(&ed.advance_text)
        } else {
            0.0
        };
        let p = &self.db.profile;
        let full = p.full_name();
        let due_days = self.db.settings.default_due_days.max(0.0) as i64;
        let due_days = if due_days == 0 { 14 } else { due_days };
        let client = &ed.client;
        Invoice {
            id: ed.editing_id.clone().unwrap_or_default(),
            invoice_number: if ed.number.trim().is_empty() { "0001".into() } else { ed.number.trim().into() },
            date: dates::iso(ed.date),
            due_date: dates::iso(ed.date + chrono::Duration::days(due_days)),
            status: if ed.status.is_empty() { "Pending".into() } else { ed.status.clone() },
            currency: Some(client.currency.clone()),
            currency_symbol: Some(client.symbol.clone()),
            from: Some(Party {
                name: if full.is_empty() { "Jarif Hasan".into() } else { full },
                location: p.address.clone(),
                email: p.email.clone(),
                ..Default::default()
            }),
            client: InvoiceClient {
                id: client.id.clone(),
                name: if client.name.is_empty() { "Client Name".into() } else { client.name.clone() },
                location: if client.is_placeholder() { String::new() } else { client.location.clone() },
                email: if client.is_placeholder() { String::new() } else { client.email.clone() },
                currency: Some(client.currency.clone()),
                currency_symbol: Some(client.symbol.clone()),
                ..Default::default()
            },
            items: ed.items.clone(),
            toggle_advance: ed.toggle_advance,
            advance_amount: advance,
            toggle_balance: ed.toggle_balance,
            total,
            balance: total - advance,
            note: if ed.note.trim().is_empty() {
                model::DEFAULT_NOTE.into()
            } else {
                ed.note.trim().into()
            },
            ..Default::default()
        }
    }

    /// Preview view for the editor (shows placeholder client details when none is selected).
    fn editor_view(&self) -> InvoiceView {
        let inv = self.editor_invoice();
        let mut v = self.view_for(&inv);
        if self.ed.client.is_placeholder() {
            v.to_location = self.ed.client.location.clone();
            v.to_email = self.ed.client.email.clone();
        }
        v
    }

    /* ---------------- Invoice lifecycle ---------------- */

    fn set_items(&mut self, items: Vec<LineItem>) {
        self.ed.price_text = items.iter().map(|i| format_price(i.price)).collect();
        self.ed.items = items;
    }

    fn start_new_invoice(&mut self, silent: bool) {
        let next = self.db.next_invoice_number();
        self.ed.editing_id = None;
        self.ed.status = "Pending".into();
        self.ed.number = next.clone();
        self.ed.date = dates::today();
        self.ed.client = CurrentClient::placeholder(self.db.settings.currency_code(), self.settings_symbol());
        self.set_items(vec![LineItem::new("Website Development", "https://websitename.com", 200.0)]);
        self.ed.toggle_advance = false;
        self.ed.advance_text = "0".into();
        self.ed.toggle_balance = false;
        self.ed.note = model::DEFAULT_NOTE.into();
        self.ed.user_modified = false;
        if !silent {
            self.toast(format!("Started new Invoice #{next}"), ToastKind::Info);
        }
    }

    fn client_from_record(&self, c: &InvoiceClient, inv: &Invoice) -> (CurrentClient, Option<String>) {
        let matched = self.db.clients.iter().find(|x| {
            (c.id.as_deref().is_some_and(|id| !id.is_empty() && x.id == id))
                || x.name.to_lowercase() == c.name.to_lowercase()
        });
        let currency = inv
            .currency
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| matched.map(|m| m.currency_code()))
            .or_else(|| c.currency.clone())
            .unwrap_or_else(|| "USD".into());
        let symbol = inv
            .currency_symbol
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| matched.map(|m| m.symbol()))
            .or_else(|| c.currency_symbol.clone())
            .unwrap_or_else(|| "$".into());
        (
            CurrentClient {
                id: c.id.clone().filter(|s| !s.is_empty()).or_else(|| matched.map(|m| m.id.clone())),
                name: if c.name.is_empty() { "Client Name".into() } else { c.name.clone() },
                location: c.location.clone(),
                email: c.email.clone(),
                currency,
                symbol,
            },
            matched.map(|m| m.id.clone()),
        )
    }

    fn load_invoice(&mut self, id: &str, silent: bool) {
        let Some(inv) = self.db.find_invoice(id).cloned() else { return };
        self.ed.editing_id = Some(inv.id.clone());
        self.ed.number = if inv.invoice_number.is_empty() { "0001".into() } else { inv.invoice_number.clone() };
        self.ed.status = inv.status_or_default().to_string();
        self.ed.date = dates::parse(&inv.date).unwrap_or_else(dates::today);
        self.ed.client = self.client_from_record(&inv.client, &inv).0;
        let items = if inv.items.is_empty() {
            vec![LineItem::new("Invoice Item", "", inv.total)]
        } else {
            inv.items.clone()
        };
        self.set_items(items);
        self.ed.user_modified = true;
        self.ed.toggle_advance = inv.toggle_advance;
        self.ed.advance_text = format_price(inv.advance_amount);
        self.ed.toggle_balance = inv.toggle_balance;
        self.ed.note = if inv.note.is_empty() { model::DEFAULT_NOTE.into() } else { inv.note.clone() };
        self.tab = Tab::Invoice;
        if !silent {
            self.toast(format!("Loaded Invoice #{} into editor", inv.invoice_number), ToastKind::Info);
        }
    }

    fn duplicate_invoice(&mut self, id: &str) {
        let Some(inv) = self.db.find_invoice(id).cloned() else { return };
        self.start_new_invoice(true);
        self.ed.client = self.client_from_record(&inv.client, &inv).0;
        self.set_items(inv.items.clone());
        self.ed.user_modified = true;
        self.ed.toggle_advance = inv.toggle_advance;
        self.ed.advance_text = format_price(inv.advance_amount);
        self.ed.toggle_balance = inv.toggle_balance;
        self.ed.note = if inv.note.is_empty() { model::DEFAULT_NOTE.into() } else { inv.note.clone() };
        self.ed.status = "Draft".into();
        self.tab = Tab::Invoice;
        let n = self.ed.number.clone();
        self.toast(format!("Duplicated invoice as #{n} (Draft)"), ToastKind::Success);
    }

    fn validate(&mut self, action: &str, now: f64) -> bool {
        if self.ed.client.is_placeholder() {
            self.toast(format!("Please select a client before {action} the invoice."), ToastKind::Warn);
            self.ed.flash_client = now + 2.0;
            self.tab = Tab::Invoice;
            return false;
        }
        let items = &self.ed.items;
        let untouched = !self.ed.user_modified
            && items.len() == 1
            && items[0].description == "Website Development"
            && items[0].sub == "https://websitename.com"
            && items[0].price == 200.0;
        let has_real = items.iter().any(|i| !i.description.trim().is_empty());
        if untouched || !has_real {
            self.toast(
                format!("Please add or customize your invoice items before {action} the invoice."),
                ToastKind::Warn,
            );
            self.ed.flash_items = now + 2.0;
            self.tab = Tab::Invoice;
            return false;
        }
        true
    }

    fn save_current(&mut self, notify: bool, now: f64) -> Option<Invoice> {
        if !self.validate(if notify { "saving" } else { "downloading" }, now) {
            return None;
        }
        let mut inv = self.editor_invoice();
        if inv.id.is_empty() {
            inv.id = format!("inv_{}", model::now_millis());
        }
        let saved = self.db.save_invoice(inv);
        self.ed.editing_id = Some(saved.id.clone());
        self.persist();
        if notify {
            self.toast(format!("Invoice #{} saved!", saved.invoice_number), ToastKind::Success);
        }
        Some(saved)
    }

    fn pdf_bytes(&self, inv: &Invoice) -> Result<Vec<u8>, String> {
        let view = self.view_for(inv);
        let page = layout::layout(&view, self.logo.clone());
        pdf::export(
            &page,
            &format!("Invoice - {}", view.number),
            &self.db.profile.pdf_password,
        )
    }

    fn export_pdf_dialog(&mut self, frame: &eframe::Frame, inv: &Invoice) {
        let number = if inv.invoice_number.is_empty() { inv.id.clone() } else { inv.invoice_number.clone() };
        let bytes = match self.pdf_bytes(inv) {
            Ok(b) => b,
            Err(e) => {
                self.toast(format!("Failed to export PDF: {e}"), ToastKind::Error);
                return;
            }
        };
        let file_name = sanitize_file_name(&format!("Invoice - {number}.pdf"));
        let mut dialog = rfd::FileDialog::new()
            .set_title("Save Invoice PDF")
            .set_file_name(&file_name)
            .add_filter("PDF Documents", &["pdf"])
            .set_parent(frame);
        if let Some(dir) = dirs::download_dir() {
            dialog = dialog.set_directory(dir);
        }
        let Some(path) = dialog.save_file() else { return };
        match std::fs::write(&path, bytes) {
            Ok(()) => self.toast(format!("Invoice #{number} saved successfully!"), ToastKind::Success),
            Err(e) => self.toast(format!("Failed to save PDF: {e}"), ToastKind::Error),
        }
    }

    fn save_text_dialog(&mut self, frame: &eframe::Frame, title: &str, name: &str, ext: &str, filter: &str, content: &str) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new()
            .set_title(title)
            .set_file_name(name)
            .add_filter(filter, &[ext])
            .set_parent(frame);
        if let Some(dir) = dirs::download_dir() {
            dialog = dialog.set_directory(dir);
        }
        let path = dialog.save_file()?;
        match std::fs::write(&path, content.as_bytes()) {
            Ok(()) => Some(path),
            Err(e) => {
                self.toast(format!("Failed to save file: {e}"), ToastKind::Error);
                None
            }
        }
    }

    /* ---------------- Action processing ---------------- */

    fn process_actions(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let now = ctx.input(|i| i.time);
        let actions = std::mem::take(&mut self.actions);
        for action in actions {
            match action {
                Action::SwitchTab(tab) => {
                    if tab == Tab::Settings && self.tab != Tab::Settings {
                        self.load_settings_form();
                    }
                    self.tab = tab;
                }
                Action::NewInvoice { silent } => {
                    self.tab = Tab::Invoice;
                    self.start_new_invoice(silent);
                }
                Action::SaveInvoice => {
                    self.save_current(true, now);
                }
                Action::DownloadCurrent => {
                    self.tab = Tab::Invoice;
                    if let Some(saved) = self.save_current(false, now) {
                        self.export_pdf_dialog(frame, &saved);
                    }
                }
                Action::SelectClient(id) => self.select_client(id, true),
                Action::LoadInvoice(id) => self.load_invoice(&id, false),
                Action::DuplicateInvoice(id) => self.duplicate_invoice(&id),
                Action::DeleteInvoice(id) => self.delete_invoice(frame, &id),
                Action::DownloadInvoice(id) => match self.db.find_invoice(&id).cloned() {
                    Some(inv) => self.export_pdf_dialog(frame, &inv),
                    None => self.toast("Invoice not found", ToastKind::Error),
                },
                Action::SetStatus(id, status) => {
                    self.db.update_invoice_status(&id, &status);
                    self.persist();
                    let editing = self.db.find_invoice(&id).map(|i| i.id.clone());
                    if editing.is_some() && editing == self.ed.editing_id {
                        self.ed.status = status.clone();
                    }
                    self.toast(format!("Status updated to {status}"), ToastKind::Info);
                }
                Action::BulkDownload => self.bulk_download(frame),
                Action::BulkDelete => self.bulk_delete(frame),
                Action::ExportCsv => self.export_csv(frame),
                Action::ExportInvoicesJson => self.export_invoices_json(frame),
                Action::UseClient(id) => {
                    self.select_client(Some(id.clone()), false);
                    self.tab = Tab::Invoice;
                    if let Some(c) = self.db.find_client(&id) {
                        let msg = format!("Loaded {} into invoice ({})", c.name, c.currency_code());
                        self.toast(msg, ToastKind::Success);
                    }
                }
                Action::EditClient(id) => self.edit_client(&id),
                Action::DeleteClient(id) => self.delete_client(frame, &id),
                Action::BulkDeleteClients => self.bulk_delete_clients(frame),
                Action::SaveClientForm => self.save_client_form(),
                Action::UploadLogo => self.upload_logo(ctx, frame),
                Action::RemoveLogo => {
                    self.db.profile.company_logo = None;
                    self.persist();
                    self.refresh_logo(ctx);
                    self.toast("Logo removed", ToastKind::Info);
                }
                Action::SaveSettings => self.save_settings_form(),
                Action::SavePdfPassword => {
                    self.db.profile.pdf_password = self.settings.pdf_password.clone();
                    self.persist();
                }
                Action::ExportDatabase => self.export_database(frame),
                Action::RestoreDatabase => self.restore_database(ctx, frame),
                Action::ResetAll => self.reset_all(ctx, frame),
                Action::OpenDataFolder => {
                    let folder = self.storage.folder();
                    let _ = std::fs::create_dir_all(&folder);
                    let _ = std::process::Command::new("explorer").arg(&folder).spawn();
                }
            }
        }
    }

    fn select_client(&mut self, id: Option<String>, notify: bool) {
        match id.and_then(|id| self.db.find_client(&id).cloned()) {
            Some(c) => {
                self.ed.client = CurrentClient {
                    id: Some(c.id.clone()),
                    name: c.name.clone(),
                    location: c.location.clone(),
                    email: c.email.clone(),
                    currency: c.currency_code(),
                    symbol: c.symbol(),
                };
                if notify {
                    self.toast(format!("Selected client: {} ({})", c.name, c.currency_code()), ToastKind::Info);
                }
            }
            None => {
                self.ed.client = CurrentClient::placeholder(self.db.settings.currency_code(), self.settings_symbol());
            }
        }
    }

    fn delete_invoice(&mut self, frame: &eframe::Frame, id: &str) {
        let Some(inv) = self.db.find_invoice(id).cloned() else { return };
        if !self.confirm(
            frame,
            "Delete invoice",
            &format!("Are you sure you want to delete Invoice #{} from the database?", inv.invoice_number),
        ) {
            return;
        }
        self.db.delete_invoice(&inv.id);
        self.hist.selected.remove(&inv.id);
        self.persist();
        if self.ed.editing_id.as_deref() == Some(inv.id.as_str()) || self.ed.number == inv.invoice_number {
            self.start_new_invoice(true);
        }
        self.toast(format!("Invoice #{} deleted", inv.invoice_number), ToastKind::Info);
    }

    fn bulk_download(&mut self, frame: &eframe::Frame) {
        let invoices: Vec<Invoice> = self
            .db
            .invoices
            .iter()
            .filter(|i| self.hist.selected.contains(&i.id))
            .cloned()
            .collect();
        if invoices.is_empty() {
            return;
        }
        let mut dialog = rfd::FileDialog::new()
            .set_title("Select Destination Folder for Invoices")
            .set_parent(frame);
        if let Some(dir) = dirs::download_dir() {
            dialog = dialog.set_directory(dir);
        }
        let Some(folder) = dialog.pick_folder() else { return };
        let mut ok = 0;
        let mut errors = Vec::new();
        for inv in &invoices {
            let number = if inv.invoice_number.is_empty() { inv.id.clone() } else { inv.invoice_number.clone() };
            let target = folder.join(sanitize_file_name(&format!("Invoice - {number}.pdf")));
            match self.pdf_bytes(inv).and_then(|b| std::fs::write(&target, b).map_err(|e| e.to_string())) {
                Ok(()) => ok += 1,
                Err(e) => errors.push(format!("#{number}: {e}")),
            }
        }
        if errors.is_empty() {
            self.toast(format!("Successfully saved {ok} invoice(s) to folder!"), ToastKind::Success);
        } else {
            self.toast(
                format!("Saved {ok} of {} invoices. Failed: {}", invoices.len(), errors.join("; ")),
                ToastKind::Error,
            );
        }
    }

    fn bulk_delete(&mut self, frame: &eframe::Frame) {
        let count = self.hist.selected.len();
        if count == 0 {
            return;
        }
        if !self.confirm(
            frame,
            "Delete invoices",
            &format!("Are you sure you want to delete {count} selected invoice(s)? This action cannot be undone."),
        ) {
            return;
        }
        let ids: Vec<String> = self.hist.selected.drain().collect();
        for id in &ids {
            self.db.delete_invoice(id);
        }
        if self.ed.editing_id.as_ref().is_some_and(|e| ids.contains(e)) {
            self.ed.editing_id = None;
        }
        self.persist();
        self.toast(format!("Deleted {count} invoice(s) successfully!"), ToastKind::Success);
    }

    fn export_csv(&mut self, frame: &eframe::Frame) {
        let list = self.filtered_invoices();
        if list.is_empty() {
            self.toast("No invoices available to export.", ToastKind::Warn);
            return;
        }
        let csv = model::invoices_to_csv(&list);
        let name = format!("invoices_export_{}.csv", dates::iso(dates::today()));
        if self.save_text_dialog(frame, "Export invoices (CSV)", &name, "csv", "CSV files", &csv).is_some() {
            self.toast("CSV export saved!", ToastKind::Success);
        }
    }

    fn export_invoices_json(&mut self, frame: &eframe::Frame) {
        let list = self.filtered_invoices();
        let json = serde_json::to_string_pretty(&list).unwrap_or_else(|_| "[]".into());
        let name = format!("invoices_export_{}.json", dates::iso(dates::today()));
        if self.save_text_dialog(frame, "Export invoices (JSON)", &name, "json", "JSON files", &json).is_some() {
            self.toast("Invoices JSON exported!", ToastKind::Success);
        }
    }

    fn export_database(&mut self, frame: &eframe::Frame) {
        let json = match serde_json::to_string_pretty(&self.db) {
            Ok(j) => j,
            Err(e) => {
                self.toast(format!("Failed to export database: {e}"), ToastKind::Error);
                return;
            }
        };
        let name = format!("jha_invoice_database_{}.json", dates::iso(dates::today()));
        if let Some(path) = self.save_text_dialog(frame, "Export JSON Database", &name, "json", "JSON Database", &json) {
            let file = path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or(name);
            self.toast(format!("JSON database backup saved ({file})"), ToastKind::Success);
        }
    }

    fn restore_database(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Select JSON Database File to Restore")
            .add_filter("JSON Database", &["json"])
            .set_parent(frame)
            .pick_file()
        else {
            return;
        };
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                self.toast(format!("Restore failed: {e}"), ToastKind::Error);
                return;
            }
        };
        let restored = match model::import_backup(&text) {
            Ok(db) => db,
            Err(e) => {
                self.toast(format!("Restore failed: {e}"), ToastKind::Error);
                return;
            }
        };
        if !self.confirm(
            frame,
            "Restore database",
            &format!(
                "Importing this JSON database will overwrite current records ({} clients, {} invoices will be replaced by {} clients and {} invoices). Proceed with restore?",
                self.db.clients.len(),
                self.db.invoices.len(),
                restored.clients.len(),
                restored.invoices.len()
            ),
        ) {
            return;
        }
        let (c, i) = (restored.clients.len(), restored.invoices.len());
        self.db = restored;
        self.persist();
        self.after_db_replaced(ctx);
        self.toast(format!("Database restored successfully! Restored {c} clients and {i} invoices."), ToastKind::Success);
    }

    fn reset_all(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if !self.confirm(
            frame,
            "Delete all data",
            "WARNING: Are you sure you want to delete all data?\n\nThis will permanently delete all invoice history, all clients, and reset all settings to defaults.",
        ) {
            return;
        }
        self.db = Database::default();
        self.persist();
        self.after_db_replaced(ctx);
        self.toast("All data deleted. The app has been reset.", ToastKind::Info);
    }

    fn after_db_replaced(&mut self, ctx: &egui::Context) {
        self.hist = HistoryState::default();
        self.clients = ClientsState::default();
        self.load_settings_form();
        self.logo_thumb = None;
        self.logo_src = Some("__reload__".into());
        self.refresh_logo(ctx);
        self.start_new_invoice(true);
    }

    /* ---------------- Top bar ---------------- */

    fn nav_tab(&mut self, ui: &mut Ui, tab: Tab, icon: &str, label: &str) {
        let active = self.tab == tab;
        let g = galley(ui, label, 13.0, if active { Weight::SemiBold } else { Weight::Medium }, TEXT_WHITE);
        let w = 14.0 + 15.0 + 7.0 + g.size().x + 14.0;
        let (rect, resp) = ui.allocate_exact_size(vec2(w, 34.0), Sense::click());
        let hovered = resp.hovered();
        let painter = ui.painter();
        if active {
            painter.add(
                egui::Shadow {
                    offset: [0, 1],
                    blur: 3,
                    spread: 0,
                    color: Color32::from_black_alpha(64),
                }
                .as_shape(rect, CornerRadius::same(8)),
            );
            painter.rect(
                rect,
                8,
                alpha(TEXT_WHITE, 0.08),
                Stroke::new(1.0, alpha(TEXT_WHITE, 0.12)),
                StrokeKind::Inside,
            );
        } else if hovered {
            painter.rect_filled(rect, 8, alpha(TEXT_WHITE, 0.04));
        }
        let (ic, tc) = if active {
            (BRAND, TEXT_WHITE)
        } else if hovered {
            (TEXT_WHITE, TEXT_WHITE)
        } else {
            (TEXT_DIM, TEXT_DIM)
        };
        let ir = Rect::from_min_size(pos2(rect.left() + 14.0, rect.center().y - 7.5), Vec2::splat(15.0));
        self.icons.paint(painter, icon, ir, ic);
        let g = galley(ui, label, 13.0, if active { Weight::SemiBold } else { Weight::Medium }, tc);
        ui.painter()
            .galley(pos2(ir.right() + 7.0, rect.center().y - g.size().y / 2.0), g, tc);
        let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
        if resp.clicked() {
            if tab == Tab::Invoice && self.tab == Tab::Invoice {
                self.actions.push(Action::NewInvoice { silent: false });
            } else {
                self.actions.push(Action::SwitchTab(tab));
            }
        }
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect().expand2(vec2(20.0, 0.0));
        ui.painter().with_clip_rect(rect).line_segment(
            [pos2(rect.left(), rect.bottom() - 0.5), pos2(rect.right(), rect.bottom() - 0.5)],
            Stroke::new(1.0, BG_SURFACE_HOVER),
        );
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            self.nav_tab(ui, Tab::Invoice, "plus", "New");
            self.nav_tab(ui, Tab::History, "chart-bar", "History");
            self.nav_tab(ui, Tab::Clients, "users", "Clients");
            self.nav_tab(ui, Tab::Settings, "gear", "Settings");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                if Btn::new(Kind::Primary, Some("file-pdf"), "Download PDF")
                    .height(36.0)
                    .show(ui, &self.icons)
                    .on_hover_text("Save and download the vector PDF")
                    .clicked()
                {
                    self.actions.push(Action::DownloadCurrent);
                }
                if Btn::new(Kind::Secondary, Some("floppy-disk"), "Save Invoice")
                    .height(36.0)
                    .show(ui, &self.icons)
                    .on_hover_text("Save this invoice to the database (Ctrl+S)")
                    .clicked()
                {
                    self.actions.push(Action::SaveInvoice);
                }
            });
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Keyboard shortcuts
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            self.actions.push(Action::SaveInvoice);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::P)) {
            self.actions.push(Action::DownloadCurrent);
        }

        egui::Panel::top("topbar")
            .exact_size(54.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(Frame::new().fill(BG_TOPBAR).inner_margin(Margin::symmetric(20, 0)))
            .show(ui, |ui| self.top_bar(ui));

        if self.tab == Tab::Invoice {
            let w = (ctx.content_rect().width() * 0.25).clamp(400.0, 440.0);
            egui::Panel::left("sidebar")
                .exact_size(w)
                .resizable(false)
                .show_separator_line(false)
                .frame(Frame::new().fill(BG_SIDEBAR))
                .show(ui, |ui| {
                    let r = ui.max_rect();
                    ui.painter().line_segment(
                        [pos2(r.right() - 0.5, r.top()), pos2(r.right() - 0.5, r.bottom())],
                        Stroke::new(1.0, BG_INPUT_HOVER),
                    );
                    self.sidebar(ui);
                });
        }

        egui::CentralPanel::default()
            .frame(Frame::new().fill(if self.tab == Tab::Invoice { BG_PREVIEW } else { BG_PREVIEW }))
            .show(ui, |ui| match self.tab {
                Tab::Invoice => self.preview_area(ui),
                Tab::History => self.history_page(ui),
                Tab::Clients => self.clients_page(ui),
                Tab::Settings => self.settings_page(ui),
            });

        show_toasts(&ctx, &self.icons, &mut self.toasts);
        self.process_actions(&ctx, frame);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        BG_APP.to_normalized_gamma_f32()
    }
}

/* ------------------------------------------------------------------------- */

pub fn format_price(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{v}");
        s
    }
}

fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|c| if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || c.is_control() { '_' } else { c })
        .collect()
}

/// Lays out content in a centred column of at most `max_w` points.
pub fn centered_page(ui: &mut Ui, max_w: f32, add: impl FnOnce(&mut Ui)) {
    let avail = ui.available_width();
    let w = (avail - 72.0).min(max_w).max(300.0);
    let side = ((avail - w) / 2.0).max(0.0);
    ui.horizontal_top(|ui| {
        ui.add_space(side);
        ui.vertical(|ui| {
            ui.set_width(w);
            ui.add_space(28.0);
            add(ui);
            ui.add_space(108.0);
        });
    });
}

/// Page header with icon, title and subtitle (+ optional right-side widget).
pub fn page_header(ui: &mut Ui, icons: &Icons, icon: &str, title: &str, subtitle: &str, right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                icons.show(ui, icon, 20.0, TEXT_WHITE);
                text(ui, title, 20.0, Weight::SemiBold, TEXT_WHITE);
            });
            text(ui, subtitle, 13.0, Weight::Regular, TEXT_DIM);
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
    });
    ui.add_space(14.0);
    hline(ui, BG_SURFACE_HOVER);
    ui.add_space(24.0);
}
