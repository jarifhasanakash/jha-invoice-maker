//! Data model for the invoice database.
//!
//! The on-disk JSON format is identical to the one used by the original
//! Electron/IndexedDB version (schema 2.1), so backups exported from either
//! version can be restored in the other. Unknown fields are preserved through
//! `extra` maps so nothing is lost on a round trip.

use chrono::{Local, SecondsFormat, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value, json};

use crate::dates;

pub const CURRENT_SCHEMA_VERSION: &str = "2.1";
pub const DEFAULT_NOTE: &str =
    "Please complete your payment within 14 days, your cooperation is greatly appreciated.";

pub const STATUSES: [&str; 4] = ["Paid", "Pending", "Draft", "Overdue"];

/* ------------------------------------------------------------------------- */
/* Lenient deserializers: old data may store numbers as strings, nulls, etc.  */
/* ------------------------------------------------------------------------- */

fn de_string<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    })
}

fn de_opt_string<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::String(s) => Some(s),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    })
}

fn de_f64<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    Ok(value_to_f64(&Value::deserialize(d)?))
}

fn de_bool<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::Bool(b) => b,
        Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        Value::String(s) => !s.is_empty() && s != "false" && s != "0",
        _ => false,
    })
}

/// JavaScript-style `parseFloat(x) || 0`.
pub fn value_to_f64(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => parse_float_prefix(s),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

/// Mimics JavaScript `parseFloat`: parses the longest numeric prefix.
pub fn parse_float_prefix(s: &str) -> f64 {
    let s = s.trim();
    let mut end = 0;
    let bytes = s.as_bytes();
    let mut seen_dot = false;
    let mut seen_digit = false;
    let mut seen_e = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'0'..=b'9' => {
                seen_digit = true;
                end = i + 1;
            }
            b'+' | b'-' if i == 0 => {}
            b'+' | b'-' if seen_e && (bytes[i - 1] == b'e' || bytes[i - 1] == b'E') => {}
            b'.' if !seen_dot && !seen_e => seen_dot = true,
            b'e' | b'E' if seen_digit && !seen_e => seen_e = true,
            _ => break,
        }
        i += 1;
    }
    if !seen_digit {
        return 0.0;
    }
    let v = s[..end].parse::<f64>().unwrap_or(0.0);
    if v.is_finite() { v } else { 0.0 }
}

fn de_platforms<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Platform>, D::Error> {
    let v = Value::deserialize(d)?;
    Ok(match v {
        Value::Array(arr) => arr
            .into_iter()
            .filter_map(|p| serde_json::from_value::<Platform>(p).ok())
            .collect(),
        _ => default_platforms(),
    })
}

fn de_invoice_client<'de, D: Deserializer<'de>>(d: D) -> Result<InvoiceClient, D::Error> {
    let v = Value::deserialize(d)?;
    Ok(match v {
        Value::String(name) => InvoiceClient {
            name,
            ..Default::default()
        },
        Value::Object(_) => serde_json::from_value(v).unwrap_or_default(),
        _ => InvoiceClient::default(),
    })
}

fn de_logo<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::String(s) if !s.trim().is_empty() => Some(s),
        _ => None,
    })
}

/* ------------------------------------------------------------------------- */
/* Schema                                                                    */
/* ------------------------------------------------------------------------- */

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Platform {
    #[serde(deserialize_with = "de_string")]
    pub id: String,
    #[serde(deserialize_with = "de_string")]
    pub name: String,
    #[serde(deserialize_with = "de_string")]
    pub email: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Platform {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            email: String::new(),
            extra: Map::new(),
        }
    }
}

fn default_platforms() -> Vec<Platform> {
    vec![
        Platform {
            id: "platform1".into(),
            name: "Payoneer".into(),
            ..Default::default()
        },
        Platform {
            id: "platform2".into(),
            name: "Wise".into(),
            ..Default::default()
        },
    ]
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Bank {
    #[serde(deserialize_with = "de_string")]
    pub bank_name: String,
    #[serde(deserialize_with = "de_string")]
    pub account_number: String,
    #[serde(deserialize_with = "de_string")]
    pub branch_name: String,
    #[serde(deserialize_with = "de_string")]
    pub branch_code: String,
    #[serde(deserialize_with = "de_string")]
    pub swift_code: String,
    #[serde(deserialize_with = "de_string")]
    pub routing_no: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    #[serde(deserialize_with = "de_string")]
    pub first_name: String,
    #[serde(deserialize_with = "de_string")]
    pub last_name: String,
    #[serde(deserialize_with = "de_string")]
    pub email: String,
    #[serde(deserialize_with = "de_string")]
    pub mobile: String,
    #[serde(deserialize_with = "de_string")]
    pub address: String,
    #[serde(deserialize_with = "de_logo")]
    pub company_logo: Option<String>,
    #[serde(deserialize_with = "de_platforms")]
    pub platforms: Vec<Platform>,
    pub bank: Bank,
    #[serde(deserialize_with = "de_string")]
    pub pdf_password: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            first_name: String::new(),
            last_name: String::new(),
            email: String::new(),
            mobile: String::new(),
            address: String::new(),
            company_logo: None,
            platforms: default_platforms(),
            bank: Bank::default(),
            pdf_password: String::new(),
            extra: Map::new(),
        }
    }
}

impl Profile {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name).trim().to_string()
    }

    /// True once the user has entered any issuer details.
    pub fn has_custom_details(&self) -> bool {
        !(self.first_name.is_empty()
            && self.last_name.is_empty()
            && self.address.is_empty()
            && self.email.is_empty())
    }

    pub fn platform(&self, idx: usize) -> Platform {
        self.platforms.get(idx).cloned().unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    #[serde(deserialize_with = "de_string")]
    pub last_invoice_number: String,
    #[serde(deserialize_with = "de_f64")]
    pub default_due_days: f64,
    #[serde(deserialize_with = "de_string")]
    pub currency_symbol: String,
    #[serde(deserialize_with = "de_opt_string", skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(deserialize_with = "de_string")]
    pub default_note: String,
    #[serde(deserialize_with = "de_bool")]
    pub auto_backup_reminder: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            last_invoice_number: "0001".into(),
            default_due_days: 14.0,
            currency_symbol: "$".into(),
            currency: None,
            default_note: DEFAULT_NOTE.into(),
            auto_backup_reminder: true,
            extra: Map::new(),
        }
    }
}

impl Settings {
    pub fn currency_code(&self) -> String {
        self.currency.clone().filter(|c| !c.is_empty()).unwrap_or_else(|| "USD".into())
    }

    pub fn symbol(&self) -> String {
        let code = self.currency_code();
        crate::currency::symbol_for(&code)
            .map(str::to_string)
            .or_else(|| (!self.currency_symbol.is_empty()).then(|| self.currency_symbol.clone()))
            .unwrap_or_else(|| "$".into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Client {
    #[serde(deserialize_with = "de_string")]
    pub id: String,
    #[serde(deserialize_with = "de_string")]
    pub name: String,
    #[serde(deserialize_with = "de_string")]
    pub location: String,
    #[serde(deserialize_with = "de_string")]
    pub email: String,
    #[serde(deserialize_with = "de_string")]
    pub phone: String,
    #[serde(deserialize_with = "de_string")]
    pub notes: String,
    #[serde(deserialize_with = "de_opt_string", skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(deserialize_with = "de_opt_string", skip_serializing_if = "Option::is_none")]
    pub currency_symbol: Option<String>,
    #[serde(deserialize_with = "de_string")]
    pub created_at: String,
    #[serde(deserialize_with = "de_string")]
    pub updated_at: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Client {
    pub fn currency_code(&self) -> String {
        self.currency.clone().filter(|c| !c.is_empty()).unwrap_or_else(|| "USD".into())
    }
    pub fn symbol(&self) -> String {
        self.currency_symbol
            .clone()
            .filter(|c| !c.is_empty())
            .or_else(|| crate::currency::symbol_for(&self.currency_code()).map(str::to_string))
            .unwrap_or_else(|| "$".into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Party {
    #[serde(deserialize_with = "de_string")]
    pub name: String,
    #[serde(deserialize_with = "de_string")]
    pub location: String,
    #[serde(deserialize_with = "de_string")]
    pub email: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct InvoiceClient {
    #[serde(deserialize_with = "de_opt_string")]
    pub id: Option<String>,
    #[serde(deserialize_with = "de_string")]
    pub name: String,
    #[serde(deserialize_with = "de_string")]
    pub location: String,
    #[serde(deserialize_with = "de_string")]
    pub email: String,
    #[serde(deserialize_with = "de_opt_string", skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(deserialize_with = "de_opt_string", skip_serializing_if = "Option::is_none")]
    pub currency_symbol: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct LineItem {
    #[serde(deserialize_with = "de_string")]
    pub description: String,
    #[serde(deserialize_with = "de_string")]
    pub sub: String,
    #[serde(deserialize_with = "de_f64")]
    pub price: f64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl LineItem {
    pub fn new(description: &str, sub: &str, price: f64) -> Self {
        Self {
            description: description.into(),
            sub: sub.into(),
            price,
            extra: Map::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Invoice {
    #[serde(deserialize_with = "de_string")]
    pub id: String,
    #[serde(deserialize_with = "de_string")]
    pub invoice_number: String,
    #[serde(deserialize_with = "de_string")]
    pub date: String,
    #[serde(deserialize_with = "de_string")]
    pub due_date: String,
    #[serde(deserialize_with = "de_string")]
    pub status: String,
    #[serde(deserialize_with = "de_opt_string", skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(deserialize_with = "de_opt_string", skip_serializing_if = "Option::is_none")]
    pub currency_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Party>,
    #[serde(deserialize_with = "de_invoice_client")]
    pub client: InvoiceClient,
    pub items: Vec<LineItem>,
    #[serde(deserialize_with = "de_bool")]
    pub toggle_advance: bool,
    #[serde(deserialize_with = "de_f64")]
    pub advance_amount: f64,
    #[serde(deserialize_with = "de_bool")]
    pub toggle_balance: bool,
    #[serde(deserialize_with = "de_f64")]
    pub total: f64,
    #[serde(deserialize_with = "de_f64")]
    pub balance: f64,
    #[serde(deserialize_with = "de_string")]
    pub note: String,
    #[serde(deserialize_with = "de_string")]
    pub created_at: String,
    #[serde(deserialize_with = "de_string")]
    pub updated_at: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Invoice {
    pub fn status_or_default(&self) -> &str {
        if self.status.is_empty() { "Pending" } else { &self.status }
    }
    pub fn symbol(&self) -> String {
        self.currency_symbol
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| self.client.currency_symbol.clone().filter(|s| !s.is_empty()))
            .unwrap_or_else(|| "$".into())
    }
    pub fn client_name(&self) -> String {
        if self.client.name.is_empty() {
            "Client".into()
        } else {
            self.client.name.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Database {
    #[serde(deserialize_with = "de_string")]
    pub version: String,
    #[serde(deserialize_with = "de_string")]
    pub schema_version: String,
    #[serde(deserialize_with = "de_f64")]
    pub data_version: f64,
    #[serde(deserialize_with = "de_string")]
    pub last_updated: String,
    pub profile: Profile,
    pub clients: Vec<Client>,
    pub invoices: Vec<Invoice>,
    pub settings: Settings,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION.into(),
            schema_version: CURRENT_SCHEMA_VERSION.into(),
            data_version: 1.0,
            last_updated: now_iso(),
            profile: Profile::default(),
            clients: Vec::new(),
            invoices: Vec::new(),
            settings: Settings::default(),
            extra: Map::new(),
        }
    }
}

impl Database {
    /// Sample data for a brand-new install, so the app doesn't start empty.
    /// Every profile/settings field is filled except the PDF password.
    pub fn demo() -> Self {
        let mut db = Database::default();
        let p = &mut db.profile;
        p.first_name = "Jarif".into();
        p.last_name = "Hasan".into();
        p.email = "jarif.hasan@example.com".into();
        p.mobile = "+880 1700-000000".into();
        p.address = "Dhaka, Bangladesh".into();
        p.platforms = vec![
            Platform { id: "platform1".into(), name: "Payoneer".into(), email: "jarif.hasan@example.com".into(), ..Default::default() },
            Platform { id: "platform2".into(), name: "Wise".into(), email: "jarif.hasan@example.com".into(), ..Default::default() },
        ];
        p.bank = Bank {
            bank_name: "Dhaka Bank".into(),
            account_number: "1234567890".into(),
            branch_name: "Banani Branch".into(),
            branch_code: "012".into(),
            swift_code: "DHAKBDDHXXX".into(),
            routing_no: "085260123".into(),
            ..Default::default()
        };
        db.settings.currency = Some("USD".into());
        db.settings.currency_symbol = "$".into();

        let now = now_iso();
        let client = Client {
            id: "client_demo".into(),
            name: "Acme Corporation".into(),
            location: "New York, USA".into(),
            email: "billing@acme.com".into(),
            phone: "+1 555-0199".into(),
            notes: "Net 14 days".into(),
            currency: Some("USD".into()),
            currency_symbol: Some("$".into()),
            created_at: now.clone(),
            updated_at: now.clone(),
            ..Default::default()
        };
        let today = Local::now().date_naive();
        let items = vec![
            LineItem::new("Website Development", "https://acme-example.com
Design, development & CMS setup", 1200.0),
            LineItem::new("Hosting & Maintenance", "12 months", 300.0),
        ];
        let total: f64 = items.iter().map(|i| i.price).sum();
        let invoice = Invoice {
            id: "inv_demo".into(),
            invoice_number: "0001".into(),
            date: crate::dates::iso(today),
            due_date: crate::dates::iso(today + chrono::Duration::days(14)),
            status: "Paid".into(),
            currency: Some("USD".into()),
            currency_symbol: Some("$".into()),
            from: Some(Party {
                name: "Jarif Hasan".into(),
                location: "Dhaka, Bangladesh".into(),
                email: "jarif.hasan@example.com".into(),
                ..Default::default()
            }),
            client: InvoiceClient {
                id: Some(client.id.clone()),
                name: client.name.clone(),
                location: client.location.clone(),
                email: client.email.clone(),
                currency: Some("USD".into()),
                currency_symbol: Some("$".into()),
                ..Default::default()
            },
            items,
            toggle_advance: true,
            advance_amount: 750.0,
            toggle_balance: true,
            total,
            balance: total - 750.0,
            note: DEFAULT_NOTE.into(),
            created_at: now.clone(),
            updated_at: now,
            ..Default::default()
        };
        db.clients.push(client);
        db.invoices.push(invoice);
        db.settings.last_invoice_number = "0001".into();
        db
    }
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub fn now_millis() -> i64 {
    Utc::now().timestamp_millis()
}

/// Zero-padded invoice number (`7` -> `0007`).
pub fn pad_number(n: i64) -> String {
    format!("{n:04}")
}

/// JavaScript-style `parseInt(str, 10)`; `None` when there is no leading integer.
pub fn parse_int_prefix(s: &str) -> Option<i64> {
    let s = s.trim();
    let (neg, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let v: i64 = digits.parse().ok()?;
    Some(if neg { -v } else { v })
}

/* ------------------------------------------------------------------------- */
/* Database operations (ported from db.js)                                    */
/* ------------------------------------------------------------------------- */

impl Database {
    pub fn touch(&mut self) {
        self.last_updated = now_iso();
    }

    pub fn find_client(&self, id_or_name: &str) -> Option<&Client> {
        self.clients
            .iter()
            .find(|c| c.id == id_or_name || c.name == id_or_name)
    }

    /// Insert or update a client (matched by id or case-insensitive name).
    pub fn save_client(&mut self, mut client: Client) -> Client {
        if client.id.is_empty() {
            client.id = format!("client_{}", now_millis());
        }
        let lname = client.name.to_lowercase();
        let now = now_iso();
        if let Some(existing) = self
            .clients
            .iter_mut()
            .find(|c| c.id == client.id || c.name.to_lowercase() == lname)
        {
            client.id = existing.id.clone();
            client.created_at = existing.created_at.clone();
            client.updated_at = now;
            let mut extra = existing.extra.clone();
            extra.extend(client.extra.clone());
            client.extra = extra;
            *existing = client.clone();
        } else {
            client.created_at = now.clone();
            client.updated_at = now;
            self.clients.push(client.clone());
        }
        self.touch();
        client
    }

    pub fn delete_client(&mut self, id: &str) {
        self.clients.retain(|c| c.id != id);
        self.touch();
    }

    pub fn find_invoice(&self, id_or_number: &str) -> Option<&Invoice> {
        self.invoices
            .iter()
            .find(|i| i.id == id_or_number || i.invoice_number == id_or_number)
    }

    /// Insert or update an invoice (matched by id or invoice number).
    pub fn save_invoice(&mut self, mut invoice: Invoice) -> Invoice {
        let now = now_iso();
        if invoice.id.is_empty() {
            invoice.id = format!("inv_{}", now_millis());
        }
        if invoice.created_at.is_empty() {
            invoice.created_at = now.clone();
        }
        invoice.updated_at = now;

        if let Some(existing) = self
            .invoices
            .iter_mut()
            .find(|i| i.id == invoice.id || i.invoice_number == invoice.invoice_number)
        {
            if !existing.created_at.is_empty() {
                invoice.created_at = existing.created_at.clone();
            }
            invoice.id = existing.id.clone();
            let mut extra = existing.extra.clone();
            extra.extend(invoice.extra.clone());
            invoice.extra = extra;
            *existing = invoice.clone();
        } else {
            self.invoices.push(invoice.clone());
        }

        if let Some(num) = parse_int_prefix(&invoice.invoice_number) {
            let current = parse_int_prefix(&self.settings.last_invoice_number).unwrap_or(0);
            if num > current {
                self.settings.last_invoice_number = pad_number(num);
            }
        }
        self.touch();
        invoice
    }

    pub fn delete_invoice(&mut self, id_or_number: &str) {
        self.invoices
            .retain(|i| i.id != id_or_number && i.invoice_number != id_or_number);
        self.touch();
    }

    pub fn update_invoice_status(&mut self, id: &str, status: &str) {
        if let Some(inv) = self
            .invoices
            .iter_mut()
            .find(|i| i.id == id || i.invoice_number == id)
        {
            inv.status = status.into();
            inv.updated_at = now_iso();
        }
        self.touch();
    }

    /// Next free invoice number, as in the original app.
    pub fn next_invoice_number(&self) -> String {
        let last_setting = parse_int_prefix(&self.settings.last_invoice_number).unwrap_or(0);
        if self.invoices.is_empty() {
            return pad_number(if last_setting > 0 { last_setting } else { 1 });
        }
        let max = self
            .invoices
            .iter()
            .filter_map(|i| parse_int_prefix(&i.invoice_number))
            .fold(0, i64::max)
            .max(last_setting);
        pad_number(max + 1)
    }
}

/* ------------------------------------------------------------------------- */
/* Migrations & import                                                        */
/* ------------------------------------------------------------------------- */

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<i64> {
        v.split('.')
            .map(|p| parse_int_prefix(p).unwrap_or(0))
            .collect()
    };
    let (pa, pb) = (parse(a), parse(b));
    let len = pa.len().max(pb.len());
    for i in 0..len {
        let x = pa.get(i).copied().unwrap_or(0);
        let y = pb.get(i).copied().unwrap_or(0);
        if x != y {
            return x.cmp(&y);
        }
    }
    std::cmp::Ordering::Equal
}

fn str_or(v: Option<&Value>, default: &str) -> String {
    match v {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => default.to_string(),
    }
}

fn is_truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
        Some(Value::String(s)) => !s.is_empty(),
        _ => true,
    }
}

fn today_iso() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

fn upgrade_to_2_1(data: &mut Map<String, Value>) {
    let stamp = now_millis();
    if let Some(Value::Array(invoices)) = data.get_mut("invoices") {
        for (idx, inv) in invoices.iter_mut().enumerate() {
            let Value::Object(o) = inv else { continue };
            let items: Vec<Value> = match o.get("items") {
                Some(Value::Array(a)) if !a.is_empty() => a
                    .iter()
                    .map(|it| {
                        json!({
                            "description": str_or(it.get("description"), ""),
                            "sub": str_or(it.get("sub"), ""),
                            "price": it.get("price").map(value_to_f64).unwrap_or(0.0),
                        })
                    })
                    .collect(),
                _ => vec![json!({
                    "description": "Website Development",
                    "sub": "",
                    "price": o.get("total").map(value_to_f64).unwrap_or(0.0),
                })],
            };
            let total = match o.get("total") {
                Some(v) if !v.is_null() => value_to_f64(v),
                _ => items
                    .iter()
                    .map(|i| i.get("price").map(value_to_f64).unwrap_or(0.0))
                    .sum(),
            };
            let advance = o.get("advanceAmount").map(value_to_f64).unwrap_or(0.0);
            let balance = match o.get("balance") {
                Some(v) if !v.is_null() => value_to_f64(v),
                _ => total - advance,
            };
            let date = str_or(o.get("date"), &today_iso());
            let client = match o.get("client").filter(|v| is_truthy(Some(v))) {
                Some(c) => c.clone(),
                None => o
                    .get("to")
                    .filter(|v| is_truthy(Some(v)))
                    .cloned()
                    .unwrap_or_else(|| json!({"name": "Client Name", "location": "", "email": ""})),
            };
            let from = o
                .get("from")
                .filter(|v| is_truthy(Some(v)))
                .cloned()
                .unwrap_or_else(|| {
                    json!({"name": "Jarif Hasan", "location": "Dhaka, Bangladesh", "email": "jarif.hasan@example.com"})
                });

            let mut out = o.clone();
            out.insert("id".into(), json!(str_or(o.get("id"), &format!("inv_{stamp}_{idx}"))));
            out.insert(
                "invoiceNumber".into(),
                json!(str_or(o.get("invoiceNumber"), &pad_number(idx as i64 + 1))),
            );
            out.insert("dueDate".into(), json!(str_or(o.get("dueDate"), &date)));
            out.insert("date".into(), json!(date));
            out.insert("status".into(), json!(str_or(o.get("status"), "Pending")));
            out.insert("from".into(), from);
            out.insert("client".into(), client);
            out.insert("items".into(), Value::Array(items));
            out.insert("toggleAdvance".into(), json!(is_truthy(o.get("toggleAdvance"))));
            out.insert("advanceAmount".into(), json!(advance));
            out.insert("toggleBalance".into(), json!(is_truthy(o.get("toggleBalance"))));
            out.insert("total".into(), json!(total));
            out.insert("balance".into(), json!(balance));
            out.insert("note".into(), json!(str_or(o.get("note"), DEFAULT_NOTE)));
            out.insert("createdAt".into(), json!(str_or(o.get("createdAt"), &now_iso())));
            out.insert("updatedAt".into(), json!(str_or(o.get("updatedAt"), &now_iso())));
            if !out.contains_key("customFields") {
                out.insert("customFields".into(), json!({}));
            }
            *o = out;
        }
    }

    if let Some(Value::Array(clients)) = data.get_mut("clients") {
        for (idx, c) in clients.iter_mut().enumerate() {
            let Value::Object(o) = c else { continue };
            let mut out = o.clone();
            out.insert("id".into(), json!(str_or(o.get("id"), &format!("client_{stamp}_{idx}"))));
            out.insert("name".into(), json!(str_or(o.get("name"), "Client Name")));
            for k in ["location", "email", "phone", "notes"] {
                out.insert(k.into(), json!(str_or(o.get(k), "")));
            }
            out.insert("createdAt".into(), json!(str_or(o.get("createdAt"), &now_iso())));
            out.insert("updatedAt".into(), json!(str_or(o.get("updatedAt"), &now_iso())));
            if !out.contains_key("customFields") {
                out.insert("customFields".into(), json!({}));
            }
            *o = out;
        }
    }

    for key in ["profile", "settings"] {
        if let Some(Value::Object(o)) = data.get_mut(key) {
            if !o.contains_key("customFields") {
                o.insert("customFields".into(), json!({}));
            }
        }
    }
}

/// Recursively merges `source` over `target`, keeping unknown source keys.
fn deep_merge(target: &Value, source: &Value) -> Value {
    match (target, source) {
        (Value::Object(t), Value::Object(s)) => {
            let mut out = t.clone();
            for (k, sv) in s {
                let merged = match (t.get(k), sv) {
                    (Some(tv @ Value::Object(_)), Value::Object(_)) => deep_merge(tv, sv),
                    _ => sv.clone(),
                };
                out.insert(k.clone(), merged);
            }
            Value::Object(out)
        }
        _ => source.clone(),
    }
}

/// Runs the schema migration pipeline on raw JSON and returns a typed database.
pub fn migrate(raw: Value) -> Result<Database, String> {
    let Value::Object(mut data) = raw else {
        return Err("Database root is not a JSON object.".into());
    };
    let version = str_or(
        data.get("schemaVersion").or_else(|| data.get("version")),
        "1.0",
    );

    if compare_versions(&version, "2.0").is_lt() {
        for key in ["profile", "settings"] {
            if !matches!(data.get(key), Some(Value::Object(_))) {
                data.insert(key.into(), json!({}));
            }
        }
        for key in ["clients", "invoices"] {
            if !matches!(data.get(key), Some(Value::Array(_))) {
                data.insert(key.into(), json!([]));
            }
        }
    }
    if compare_versions(&version, "2.1").is_lt() {
        upgrade_to_2_1(&mut data);
    }

    let data_version = data.get("dataVersion").map(value_to_f64).unwrap_or(1.0);
    let defaults = serde_json::to_value(Database::default()).map_err(|e| e.to_string())?;
    let mut merged = deep_merge(&defaults, &Value::Object(data));
    if let Value::Object(m) = &mut merged {
        m.insert("schemaVersion".into(), json!(CURRENT_SCHEMA_VERSION));
        m.insert("version".into(), json!(CURRENT_SCHEMA_VERSION));
        m.insert("dataVersion".into(), json!(data_version.max(1.0) + 1.0));
        m.insert("lastUpdated".into(), json!(now_iso()));
    }
    let mut db: Database = serde_json::from_value(merged).map_err(|e| e.to_string())?;
    if db.profile.platforms.is_empty() {
        db.profile.platforms = default_platforms();
    }
    Ok(db)
}

fn maybe_parse_string(v: &Value) -> Value {
    match v {
        Value::String(s) => serde_json::from_str(s).unwrap_or(Value::Null),
        other => other.clone(),
    }
}

/// Parses a JSON backup (current schema or the legacy v1 export format).
pub fn import_backup(text: &str) -> Result<Database, String> {
    let parsed: Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|_| "Invalid JSON format: file is corrupted or not a valid JSON document.".to_string())?;

    let has = |k: &str| parsed.get(k).map(|v| is_truthy(Some(v))).unwrap_or(false);
    if has("profile") || has("clients") || has("invoices") {
        return migrate(parsed);
    }
    if !(has("bankDetails") || has("clientList") || has("invoiceHistory")) {
        return Err(
            "Unrecognized database format: it contains no profile, clients or invoices.".into(),
        );
    }

    // Legacy v1 export.
    let mut legacy = serde_json::to_value(Database::default()).map_err(|e| e.to_string())?;
    let today = today_iso();
    if let Some(b) = parsed.get("bankDetails").map(maybe_parse_string) {
        let p = &mut legacy["profile"];
        for (k, src) in [
            ("firstName", "firstName"),
            ("lastName", "lastName"),
            ("email", "email"),
            ("mobile", "mobile"),
            ("address", "address"),
        ] {
            if let Some(s) = b.get(src).and_then(Value::as_str).filter(|s| !s.is_empty()) {
                p[k] = json!(s);
            }
        }
        let mut bank = Map::new();
        for k in ["bankName", "accountNumber", "branchName", "branchCode", "swiftCode", "routingNo"] {
            bank.insert(k.into(), json!(str_or(b.get(k), "")));
        }
        p["bank"] = Value::Object(bank);
    }
    if let Some(logo) = parsed.get("companyLogo").and_then(Value::as_str) {
        legacy["profile"]["companyLogo"] = json!(logo);
    }
    if let Some(Value::Array(list)) = parsed.get("clientList").map(maybe_parse_string) {
        legacy["clients"] = Value::Array(
            list.iter()
                .enumerate()
                .map(|(i, c)| {
                    json!({
                        "id": format!("client_{i}"),
                        "name": str_or(c.get("name"), ""),
                        "location": str_or(c.get("location"), ""),
                        "email": str_or(c.get("email"), ""),
                        "phone": "",
                        "notes": "",
                    })
                })
                .collect(),
        );
    }
    if let Some(Value::Array(list)) = parsed.get("invoiceHistory").map(maybe_parse_string) {
        legacy["invoices"] = Value::Array(
            list.iter()
                .enumerate()
                .map(|(i, h)| {
                    let amount = h.get("amount").map(value_to_f64).unwrap_or(0.0);
                    let date = str_or(h.get("date"), &today);
                    json!({
                        "id": format!("inv_{i}"),
                        "invoiceNumber": str_or(h.get("number"), &pad_number(i as i64 + 1)),
                        "date": date,
                        "dueDate": date,
                        "status": "Paid",
                        "client": { "name": str_or(h.get("client"), ""), "location": "", "email": "" },
                        "items": [{ "description": "Website Development", "sub": "", "price": amount }],
                        "toggleAdvance": false,
                        "advanceAmount": 0,
                        "toggleBalance": false,
                        "total": amount,
                        "balance": amount,
                        "note": DEFAULT_NOTE,
                        "createdAt": str_or(h.get("timestamp"), &now_iso()),
                        "updatedAt": str_or(h.get("timestamp"), &now_iso()),
                    })
                })
                .collect(),
        );
    }
    if let Some(n) = parsed.get("lastInvoiceNumber") {
        legacy["settings"]["lastInvoiceNumber"] = json!(str_or(Some(n), "0001"));
    }
    // The defaults carry schema 2.1 already; force the invoice/client normalisation.
    if let Value::Object(m) = &mut legacy {
        m.insert("schemaVersion".into(), json!("2.0"));
        m.insert("version".into(), json!("2.0"));
    }
    migrate(legacy)
}

/// CSV export (same columns as the original app).
pub fn invoices_to_csv(list: &[&Invoice]) -> String {
    let q = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
    let num = |v: f64| {
        if v.fract() == 0.0 && v.abs() < 1e15 {
            format!("{}", v as i64)
        } else {
            format!("{v}")
        }
    };
    let mut csv =
        String::from("Invoice Number,Date,Due Date,Status,Client Name,Client Email,Amount,Advance,Balance\n");
    for inv in list {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            q(&inv.invoice_number),
            q(&inv.date),
            q(&inv.due_date),
            q(inv.status_or_default()),
            q(&inv.client.name),
            q(&inv.client.email),
            num(inv.total),
            num(inv.advance_amount),
            num(inv.balance),
        ));
    }
    csv
}

/// Sorts invoices newest first (date, then invoice number), like the history view.
pub fn sort_invoices_desc(list: &mut [&Invoice]) {
    list.sort_by(|a, b| {
        let da = dates::parse(&a.date);
        let db = dates::parse(&b.date);
        db.cmp(&da).then_with(|| b.invoice_number.cmp(&a.invoice_number))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_float_matches_js() {
        assert_eq!(parse_float_prefix("12.5abc"), 12.5);
        assert_eq!(parse_float_prefix("abc"), 0.0);
        assert_eq!(parse_float_prefix("-3"), -3.0);
        assert_eq!(parse_float_prefix("1e3"), 1000.0);
        assert_eq!(parse_float_prefix(""), 0.0);
    }

    #[test]
    fn migrates_old_invoice_and_preserves_unknown_fields() {
        let raw = json!({
            "version": "2.0",
            "profile": {"firstName": "Jarif", "myCustom": 5},
            "clients": [{"name": "Acme"}],
            "invoices": [{"invoiceNumber": "0007", "total": "120", "client": "Acme", "items": [{"description": "Web", "price": "100"}]}],
            "settings": {"lastInvoiceNumber": "0007"},
            "somethingNew": true
        });
        let db = migrate(raw).unwrap();
        assert_eq!(db.profile.first_name, "Jarif");
        assert_eq!(db.profile.extra.get("myCustom"), Some(&json!(5)));
        assert_eq!(db.extra.get("somethingNew"), Some(&json!(true)));
        assert_eq!(db.invoices[0].total, 120.0);
        assert_eq!(db.invoices[0].items[0].price, 100.0);
        assert_eq!(db.invoices[0].client.name, "Acme");
        assert_eq!(db.profile.platforms.len(), 2);
        assert_eq!(db.next_invoice_number(), "0008");
        // Round trip through JSON keeps everything.
        let text = serde_json::to_string(&db).unwrap();
        let again = import_backup(&text).unwrap();
        assert_eq!(again.invoices.len(), 1);
        assert_eq!(again.profile.extra.get("myCustom"), Some(&json!(5)));
    }

    #[test]
    fn imports_legacy_v1() {
        let raw = json!({
            "bankDetails": "{\"firstName\":\"Jarif\",\"bankName\":\"Dhaka Bank\"}",
            "clientList": [{"name": "Acme", "location": "NY", "email": "a@b.co"}],
            "invoiceHistory": [{"number": "0003", "client": "Acme", "amount": 300, "date": "2026-01-02"}],
            "lastInvoiceNumber": "0003"
        });
        let db = import_backup(&raw.to_string()).unwrap();
        assert_eq!(db.profile.bank.bank_name, "Dhaka Bank");
        assert_eq!(db.clients[0].name, "Acme");
        assert_eq!(db.invoices[0].total, 300.0);
        assert_eq!(db.invoices[0].status, "Paid");
    }

    #[test]
    fn demo_data_is_complete() {
        let db = Database::demo();
        assert_eq!(db.clients.len(), 1);
        assert_eq!(db.invoices.len(), 1);
        assert!(db.profile.pdf_password.is_empty());
        assert!(!db.profile.bank.routing_no.is_empty());
        assert_eq!(db.next_invoice_number(), "0002");
        // Survives a save/load round trip.
        let again = import_backup(&serde_json::to_string(&db).unwrap()).unwrap();
        assert_eq!(again.invoices[0].client.name, "Acme Corporation");
    }

    #[test]
    fn save_invoice_updates_last_number() {
        let mut db = Database::default();
        assert_eq!(db.next_invoice_number(), "0001");
        let inv = Invoice {
            invoice_number: "0042".into(),
            ..Default::default()
        };
        db.save_invoice(inv);
        assert_eq!(db.settings.last_invoice_number, "0042");
        assert_eq!(db.next_invoice_number(), "0043");
    }
}
