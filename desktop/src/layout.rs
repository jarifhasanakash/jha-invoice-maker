//! Invoice page layout.
//!
//! This is a direct port of the original invoice CSS (1080 px wide paper,
//! Lexend type scale, 96 px padding, rounded tables, footer band). It produces
//! a display list in CSS pixels that is rendered both on screen (raster preview)
//! and into the PDF (vector), so the preview always matches the export.

use std::sync::Arc;

use crate::fonts::{self, ASCENT, NORMAL_LINE_HEIGHT, ShapedText, Weight};
use crate::logo::Logo;
use crate::model::LineItem;

pub type Rgb = [u8; 3];

pub const PAPER_WIDTH: f32 = 1080.0;
pub const PAPER_MIN_HEIGHT: f32 = 1122.0;

const PAD: f32 = 96.0;
const CONTENT_W: f32 = PAPER_WIDTH - 2.0 * PAD;

const C_PAPER: Rgb = [0xf5, 0xf5, 0xf5];
const C_TEXT: Rgb = [0x27, 0x28, 0x29];
const C_MUTED: Rgb = [0x4d, 0x4d, 0x4d];
const C_LIGHT: Rgb = [0x99, 0x99, 0x99];
const C_BORDER: Rgb = [0xdf, 0xe4, 0xea];
const C_N50: Rgb = [0xf9, 0xf9, 0xfa];
const C_BRAND: Rgb = [0xea, 0x4e, 0x25];

pub enum Paint {
    /// Rounded rectangle; `stroke` is a 1 px line centred on the rectangle edge.
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radius: f32,
        fill: Option<Rgb>,
        stroke: Option<Rgb>,
    },
    Text {
        x: f32,
        baseline: f32,
        color: Rgb,
        shaped: ShapedText,
    },
    Logo {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
}

pub struct PageLayout {
    pub width: f32,
    pub height: f32,
    pub background: Rgb,
    pub items: Vec<Paint>,
    pub logo: Arc<Logo>,
}

/// Everything that appears on the printed invoice.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InvoiceView {
    pub number: String,
    pub date: String,
    pub client_name: String,
    pub to_location: String,
    pub to_email: String,
    pub from_name: String,
    pub from_location: String,
    pub from_email: String,
    pub items: Vec<LineItem>,
    pub symbol: String,
    pub show_advance: bool,
    pub show_balance: bool,
    pub advance: f64,
    pub note: String,
    pub platforms: [(String, String); 2],
    pub bank_left: Vec<(String, String)>,
    pub bank_right: Vec<(String, String)>,
    pub signature_name: String,
    pub mobile: String,
}

impl InvoiceView {
    pub fn total(&self) -> f64 {
        self.items.iter().map(|i| i.price).sum()
    }
}

/// JavaScript `Number.prototype.toFixed(2)`.
/// Two decimals with thousands separators: `1500` -> `1,500.00`.
pub fn fixed2(v: f64) -> String {
    let s = format!("{:.2}", v);
    let s = if s == "-0.00" { "0.00".to_string() } else { s };
    let (sign, rest) = match s.strip_prefix('-') {
        Some(r) => ("-", r),
        None => ("", s.as_str()),
    };
    let (int, frac) = rest.split_once('.').unwrap_or((rest, "00"));
    let mut grouped = String::new();
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    format!("{sign}{grouped}.{frac}")
}

struct Builder {
    items: Vec<Paint>,
}

#[derive(Clone, Copy)]
struct Style {
    weight: Weight,
    size: f32,
    color: Rgb,
    /// Explicit line height in px; `None` = CSS `normal`.
    line_height: Option<f32>,
}

impl Style {
    const fn new(weight: Weight, size: f32, color: Rgb) -> Self {
        Self {
            weight,
            size,
            color,
            line_height: None,
        }
    }
    const fn lh(mut self, lh: f32) -> Self {
        self.line_height = Some(lh);
        self
    }
    fn line_h(&self) -> f32 {
        self.line_height.unwrap_or(self.size * NORMAL_LINE_HEIGHT)
    }
    /// Distance from the top of a line box to the baseline.
    fn baseline_offset(&self) -> f32 {
        (self.line_h() - self.size * NORMAL_LINE_HEIGHT) / 2.0 + self.size * ASCENT
    }
}

fn width(text: &str, st: Style) -> f32 {
    fonts::book().width(text, st.weight, st.size)
}

/// Collapses whitespace like CSS `white-space: normal`.
fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Greedy line breaking with `overflow-wrap: break-word`.
fn wrap(text: &str, st: Style, max_w: f32, pre_wrap: bool) -> Vec<String> {
    let paragraphs: Vec<String> = if pre_wrap {
        let t = text.replace("\r\n", "\n");
        let t = t.strip_suffix('\n').unwrap_or(&t).to_string();
        t.split('\n').map(str::to_string).collect()
    } else {
        let c = collapse_ws(text);
        if c.is_empty() {
            return Vec::new();
        }
        vec![c]
    };

    let mut lines = Vec::new();
    for para in paragraphs {
        if para.trim().is_empty() {
            lines.push(para);
            continue;
        }
        // Tokens: words with their preceding whitespace.
        let mut tokens: Vec<(String, String)> = Vec::new();
        let mut ws = String::new();
        let mut word = String::new();
        for ch in para.chars() {
            if ch == ' ' || ch == '\t' {
                if !word.is_empty() {
                    tokens.push((std::mem::take(&mut ws), std::mem::take(&mut word)));
                }
                ws.push(if ch == '\t' { ' ' } else { ch });
            } else {
                word.push(ch);
            }
        }
        if !word.is_empty() {
            tokens.push((ws, word));
        }

        let mut line = String::new();
        for (space, word) in tokens {
            let candidate = if line.is_empty() && !pre_wrap {
                word.clone()
            } else {
                format!("{line}{space}{word}")
            };
            if width(candidate.trim_end(), st) <= max_w + 0.01 || line.trim().is_empty() {
                if width(candidate.trim_end(), st) > max_w + 0.01 {
                    // A single word longer than the line: break it by characters.
                    let mut cur = if pre_wrap { format!("{line}{space}") } else { String::new() };
                    for ch in word.chars() {
                        let next = format!("{cur}{ch}");
                        if width(&next, st) > max_w + 0.01 && !cur.trim().is_empty() {
                            lines.push(std::mem::take(&mut cur));
                            cur.push(ch);
                        } else {
                            cur = next;
                        }
                    }
                    line = cur;
                } else {
                    line = candidate;
                }
            } else {
                lines.push(line.trim_end().to_string());
                // Re-run the word on an empty line (handles over-long words).
                let w = width(&word, st);
                if w > max_w + 0.01 {
                    let mut cur = String::new();
                    for ch in word.chars() {
                        let next = format!("{cur}{ch}");
                        if width(&next, st) > max_w + 0.01 && !cur.is_empty() {
                            lines.push(std::mem::take(&mut cur));
                            cur.push(ch);
                        } else {
                            cur = next;
                        }
                    }
                    line = cur;
                } else {
                    line = word;
                }
            }
        }
        lines.push(line.trim_end().to_string());
    }
    lines
}

impl Builder {
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, fill: Option<Rgb>, stroke: Option<Rgb>) {
        self.items.push(Paint::Rect {
            x,
            y,
            w,
            h,
            radius,
            fill,
            stroke,
        });
    }

    /// Solid 1 px (or `h` px) bar.
    fn bar(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgb) {
        self.rect(x, y, w, h, 0.0, Some(color), None);
    }

    /// Draws one line of text with its line box starting at `top`.
    fn line(&mut self, x: f32, top: f32, text: &str, st: Style) -> f32 {
        if !text.is_empty() {
            let shaped = fonts::book().shape(text, st.weight, st.size);
            let w = shaped.width;
            self.items.push(Paint::Text {
                x,
                baseline: top + st.baseline_offset(),
                color: st.color,
                shaped,
            });
            w
        } else {
            0.0
        }
    }

    /// Right-aligned single line ending at `right`.
    fn line_right(&mut self, right: f32, top: f32, text: &str, st: Style) {
        let w = width(text, st);
        self.line(right - w, top, text, st);
    }

    /// Wrapped block of text. Returns its height (0 for empty text).
    fn block(&mut self, x: f32, top: f32, w: f32, text: &str, st: Style, pre_wrap: bool) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let lines = wrap(text, st, w, pre_wrap);
        let lh = st.line_h();
        for (i, l) in lines.iter().enumerate() {
            self.line(x, top + i as f32 * lh, l, st);
        }
        lines.len() as f32 * lh
    }

    fn block_height(text: &str, w: f32, st: Style, pre_wrap: bool) -> f32 {
        if text.is_empty() {
            0.0
        } else {
            wrap(text, st, w, pre_wrap).len() as f32 * st.line_h()
        }
    }

    /// Amount cell: `[-]` `symbol` `value` separated by 4 px, right-aligned.
    fn amount_right(&mut self, right: f32, top: f32, parts: &[&str], st: Style) {
        let parts: Vec<&str> = parts.iter().copied().filter(|p| !p.is_empty()).collect();
        let total: f32 = parts.iter().map(|p| width(p, st)).sum::<f32>()
            + 4.0 * (parts.len().saturating_sub(1)) as f32;
        let mut x = right - total;
        for p in parts {
            x += self.line(x, top, p, st) + 4.0;
        }
    }
}

pub fn layout(view: &InvoiceView, logo: Arc<Logo>) -> PageLayout {
    let mut b = Builder { items: Vec::new() };
    let x0 = PAD;
    let right = PAD + CONTENT_W;

    /* ---------------- Header ---------------- */
    let mut y = PAD;
    b.items.push(Paint::Logo {
        x: x0,
        y,
        w: 140.0,
        h: 140.0,
    });

    let mx = x0 + 140.0 + 112.0;
    let mw = CONTENT_W - 140.0 - 112.0;

    // Title row: "Invoice | Client Name" (flex-wrap, baseline aligned, gap 16).
    let title = Style::new(Weight::Regular, 36.0, C_TEXT).lh(36.0);
    let sep = Style::new(Weight::Regular, 36.0, C_TEXT);
    let name_st = Style::new(Weight::Regular, 36.0, C_TEXT).lh(36.0 * 1.15);
    let w_inv = width("Invoice", title);
    let w_sep = width("|", sep);
    let name_x = mx + w_inv + 16.0 + w_sep + 16.0;
    let avail = mx + mw - name_x;
    let client_name = if view.client_name.is_empty() { "Client Name" } else { &view.client_name };
    let name_w = width(&collapse_ws(client_name), name_st);

    let b_inv = title.baseline_offset();
    let b_sep = -4.0 + sep.baseline_offset(); // margin-top: -4px
    let b_name = name_st.baseline_offset();
    let sep_box_h = sep.line_h() - 4.0;

    let title_h;
    if name_w <= avail + 0.01 {
        let base = b_inv.max(b_sep).max(b_name);
        b.line(mx, y + base - b_inv, "Invoice", title);
        b.line(mx + w_inv + 16.0, y + base - sep.baseline_offset(), "|", sep);
        b.line(name_x, y + base - b_name, &collapse_ws(client_name), name_st);
        title_h = (base - b_inv + title.line_h())
            .max(base - b_sep + sep_box_h)
            .max(base - b_name + name_st.line_h());
    } else {
        let base = b_inv.max(b_sep);
        b.line(mx, y + base - b_inv, "Invoice", title);
        b.line(mx + w_inv + 16.0, y + base - sep.baseline_offset(), "|", sep);
        let first_h = (base - b_inv + title.line_h()).max(base - b_sep + sep_box_h);
        let name_top = y + first_h + 16.0;
        let h = b.block(mx, name_top, mw, client_name, name_st, false);
        title_h = first_h + 16.0 + h;
    }

    // Meta row: "No. 0001 | Date: 16-Sep-2026"
    let my = y + title_h + 16.0;
    let label = Style::new(Weight::Medium, 22.0, C_LIGHT).lh(22.0);
    let value = Style::new(Weight::Light, 22.0, C_LIGHT).lh(22.0);
    let meta_sep = Style::new(Weight::Regular, 22.0, C_LIGHT);
    let mut x = mx;
    x += b.line(x, my, "No.", label) + 8.0;
    x += b.line(x, my, &view.number, value) + 12.0;
    x += b.line(x, my - 4.0, "|", meta_sep) + 12.0;
    x += b.line(x, my, "Date:", label) + 8.0;
    b.line(x, my, &view.date, value);
    let meta_h = (label.line_h()).max(meta_sep.line_h() - 4.0);

    // Parties
    let py = my + meta_h + 16.0 + 32.0;
    let col_w = (mw - 40.0) / 2.0;
    let party_h3 = Style::new(Weight::Medium, 28.0, C_TEXT);
    let party_name = Style::new(Weight::Medium, 18.0, C_TEXT).lh(25.0);
    let party_sub = Style::new(Weight::Light, 18.0, C_MUTED).lh(25.0);
    let party = |b: &mut Builder, px: f32, heading: &str, lines: [&str; 3]| -> f32 {
        b.line(px, py, heading, party_h3);
        let mut yy = py + party_h3.line_h() + 12.0;
        let mut first = true;
        for (i, text) in lines.iter().enumerate() {
            if text.trim().is_empty() {
                continue;
            }
            if !first {
                yy += 2.0;
            }
            first = false;
            let (st, pre) = if i == 0 { (party_name, false) } else { (party_sub, true) };
            yy += b.block(px, yy, col_w, text, st, pre);
        }
        yy - py
    };
    let from_h = party(
        &mut b,
        mx,
        "From",
        [&view.from_name, &view.from_location, &view.from_email],
    );
    let to_h = party(
        &mut b,
        mx + col_w + 40.0,
        "To",
        [client_name, &view.to_location, &view.to_email],
    );
    let meta_total = (py - y) + from_h.max(to_h);
    y += meta_total.max(140.0) + 64.0;

    /* ---------------- Items table ---------------- */
    let table_y = y;
    let th = Style::new(Weight::SemiBold, 20.0, C_TEXT);
    let inner_l = x0 + 30.0;
    let inner_r = right - 30.0;
    let amount_l = inner_r - 224.0;
    let divider_x = amount_l - 30.0 - 1.0;
    let desc_w = divider_x - 30.0 - inner_l;

    // Header (min-height 88 incl. 1px bottom border)
    let head_content = 87.0;
    b.line(inner_l, y + (head_content - th.line_h()) / 2.0, "DESCRIPTION", th);
    b.line_right(inner_r, y + (head_content - th.line_h()) / 2.0, "AMOUNT", th);
    b.bar(divider_x, y, 1.0, head_content, C_BORDER);
    b.bar(x0, y + head_content, CONTENT_W, 1.0, C_BORDER);
    y += 88.0;

    let item_name = Style::new(Weight::Medium, 18.0, C_TEXT).lh(24.0);
    let item_sub = Style::new(Weight::Light, 18.0, C_MUTED).lh(24.0);
    let price_st = Style::new(Weight::Light, 18.0, C_TEXT).lh(24.0);
    let n_items = view.items.len();
    for (i, item) in view.items.iter().enumerate() {
        let last = i + 1 == n_items;
        let border: f32 = if last { 0.0 } else { 1.0 };
        let name_h = Builder::block_height(&item.description, desc_w, item_name, false);
        let sub_h = Builder::block_height(&item.sub, desc_w, item_sub, true);
        let content = 40.0 + name_h + sub_h;
        let content_box = (88.0 - border).max(content);
        let desc_top = y + (content_box - content) / 2.0 + 20.0;
        b.block(inner_l, desc_top, desc_w, &item.description, item_name, false);
        b.block(inner_l, desc_top + name_h, desc_w, &item.sub, item_sub, true);

        let price = item.price;
        let abs = fixed2(price.abs());
        let neg = if price < 0.0 { "-" } else { "" };
        b.amount_right(
            inner_r,
            y + (content_box - price_st.line_h()) / 2.0,
            &[neg, &view.symbol, &abs],
            price_st,
        );
        b.bar(divider_x, y, 1.0, content_box, C_BORDER);
        if !last {
            b.bar(x0, y + content_box, CONTENT_W, 1.0, C_BORDER);
        }
        y += content_box + border;
    }
    // Table outline (outline-offset: -1px → drawn inside the box).
    b.rect(x0 + 0.5, table_y + 0.5, CONTENT_W - 1.0, y - table_y - 1.0, 19.5, None, Some(C_BORDER));

    // Total
    y += 20.0;
    let total_st = Style::new(Weight::SemiBold, 20.0, C_TEXT);
    let total_h = 32.0 * 2.0 + total_st.line_h();
    b.rect(x0, y, CONTENT_W, total_h, 20.0, Some(C_N50), None);
    b.rect(x0 - 0.5, y - 0.5, CONTENT_W + 1.0, total_h + 1.0, 20.5, None, Some(C_BORDER));
    b.line(inner_l, y + 32.0, "TOTAL", total_st);
    let total = view.total();
    let total_text = format!("{}{}", if total < 0.0 { "-" } else { "" }, fixed2(total.abs()));
    b.amount_right(
        inner_r,
        y + 32.0,
        &[&view.symbol, &total_text],
        Style { color: C_BRAND, ..total_st },
    );
    y += total_h;

    // Payment breakdown
    if view.show_advance || view.show_balance {
        y += 20.0;
        let lab = Style::new(Weight::Medium, 18.0, C_TEXT);
        let amt = Style::new(Weight::Light, 18.0, C_TEXT);
        let row_h = 20.0 * 2.0 + lab.line_h();
        let both = view.show_advance && view.show_balance;
        let h = 2.0 + row_h * if both { 2.0 } else { 1.0 } + if both { 1.0 } else { 0.0 };
        b.rect(x0, y, CONTENT_W, h, 20.0, Some(C_N50), None);
        b.rect(x0 + 0.5, y + 0.5, CONTENT_W - 1.0, h - 1.0, 19.5, None, Some(C_BORDER));
        let advance = if view.show_advance { view.advance } else { 0.0 };
        let balance = total - advance;
        let mut ry = y + 1.0;
        let (l, r) = (x0 + 1.0 + 30.0, right - 1.0 - 30.0);
        if view.show_advance {
            b.line(l, ry + 20.0, "Pay in Advance", lab);
            b.amount_right(r, ry + 20.0, &[&view.symbol, &fixed2(advance)], amt);
            ry += row_h;
            if both {
                b.bar(x0 + 1.0, ry, CONTENT_W - 2.0, 1.0, C_BORDER);
                ry += 1.0;
            }
        }
        if view.show_balance {
            b.line(l, ry + 20.0, "Pay upon Completion", lab);
            b.amount_right(r, ry + 20.0, &[&view.symbol, &fixed2(balance)], amt);
        }
        y += h;
    }

    // Note
    y += 20.0;
    let note_lbl = Style::new(Weight::Light, 18.0, C_LIGHT).lh(25.2);
    let note_txt = Style::new(Weight::Light, 18.0, C_MUTED).lh(25.2);
    let w_note = b.line(x0, y, "Note", note_lbl);
    let w_colon = b.line(x0 + w_note + 16.0, y, ":", note_lbl);
    let tx = x0 + w_note + 16.0 + w_colon + 16.0;
    let note = if view.note.trim().is_empty() { crate::model::DEFAULT_NOTE } else { &view.note };
    let nh = b.block(tx, y, right - tx, note, note_txt, false).max(note_lbl.line_h());
    y += nh + 40.0;

    /* ---------------- Payment info ---------------- */
    y += 64.0;
    let h4 = Style::new(Weight::Medium, 18.0, C_LIGHT);
    let lbl = Style::new(Weight::Light, 18.0, C_LIGHT);
    let val = Style::new(Weight::Light, 18.0, C_MUTED);
    let row_lh = lbl.line_h();

    let visible_platforms: Vec<&(String, String)> = view
        .platforms
        .iter()
        .filter(|(n, e)| !(n.trim().is_empty() && e.trim().is_empty()))
        .collect();
    let bank_left: Vec<&(String, String)> =
        view.bank_left.iter().filter(|(_, v)| !v.trim().is_empty()).collect();
    let bank_right: Vec<&(String, String)> =
        view.bank_right.iter().filter(|(_, v)| !v.trim().is_empty()).collect();
    let any_bank = !bank_left.is_empty() || !bank_right.is_empty();
    let any_pm = !visible_platforms.is_empty();

    // A label/value row (".row"): label column, ":" column, wrapped value.
    let kv_row = |b: &mut Builder, x: f32, top: f32, w: f32, lbl_w: f32, k: &str, v: &str| -> f32 {
        let vx = x + lbl_w + 16.0 + 10.0 + 16.0;
        let vw = x + w - vx;
        let vh = Builder::block_height(v, vw, val, false).max(row_lh);
        b.line(x, top + (vh - row_lh) / 2.0, k, lbl);
        let cw = width(":", lbl);
        b.line(x + lbl_w + 16.0 + (10.0 - cw) / 2.0, top + (vh - row_lh) / 2.0, ":", lbl);
        b.block(vx, top, vw, v, val, false);
        vh
    };

    if any_pm || any_bank {
        b.line(x0, y, "Payment info", Style::new(Weight::Medium, 28.0, C_TEXT));
        y += 28.0 * NORMAL_LINE_HEIGHT + 12.0 + 28.0;
    }

    // Payment methods (always present, may be empty).
    let pm_h = if any_pm {
        let n = visible_platforms.len() as f32;
        let col_w = (CONTENT_W - 40.0 * (n - 1.0)) / n;
        let mut max_h: f32 = 0.0;
        for (i, (name, email)) in visible_platforms.iter().enumerate() {
            let cx = x0 + i as f32 * (col_w + 40.0);
            let mut h = 8.0; // h4 margin-bottom
            if !name.trim().is_empty() {
                b.line(cx, y, name, h4);
                h += h4.line_h();
            }
            if !email.trim().is_empty() {
                h += 4.0;
                h += kv_row(&mut b, cx, y + h, col_w, 60.0, "Email", email);
            }
            max_h = max_h.max(h);
        }
        max_h
    } else {
        0.0
    };
    y += pm_h + 28.0;

    // Divider
    b.bar(x0, y, CONTENT_W, 1.0, C_BORDER);
    y += 1.0;

    if any_bank {
        y += 28.0;
        let col_w = (CONTENT_W - 40.0) / 2.0;
        b.line(x0, y, "Bank", h4);
        let mut lh = h4.line_h() + 8.0;
        for (k, v) in &bank_left {
            lh += 4.0;
            lh += kv_row(&mut b, x0, y + lh, col_w, 140.0, k, v);
        }
        let rx = x0 + col_w + 40.0;
        let mut rh: f32 = 0.0;
        for (i, (k, v)) in bank_right.iter().enumerate() {
            if i > 0 {
                rh += 4.0;
            }
            rh += kv_row(&mut b, rx, y + rh, col_w, 140.0, k, v);
        }
        y += lh.max(rh);
    }

    /* ---------------- Thank-you footer ---------------- */
    let footer_h = 1.0 + 48.0 + 28.0 + 12.0 + 20.0 + 48.0;
    let main_h = y + PAD;
    let height = (main_h + footer_h).max(PAPER_MIN_HEIGHT).ceil();
    let fy = height - footer_h;
    b.rect(0.0, fy, PAPER_WIDTH, footer_h, 0.0, Some(C_N50), None);
    b.bar(0.0, fy, PAPER_WIDTH, 1.0, C_BORDER);
    let ty = fy + 1.0 + 48.0;
    b.line(100.0, ty, "Thank you!", Style::new(Weight::Medium, 28.0, C_TEXT).lh(28.0));

    // Signature line: inline spans in a 16px "normal" line box (baseline at 16px).
    let sig_top = ty + 28.0 + 12.0;
    let sig_baseline = sig_top + 16.0;
    let sig_name = Style::new(Weight::Medium, 18.0, C_LIGHT).lh(18.0);
    let sig_text = Style::new(Weight::Regular, 18.0, C_LIGHT).lh(18.0);
    let space = width(" ", Style::new(Weight::Regular, 16.0, C_LIGHT));
    let mut sx = 100.0;
    let top_for = |st: Style| sig_baseline - st.baseline_offset();
    if !view.signature_name.is_empty() {
        sx += b.line(sx, top_for(sig_name), &view.signature_name, sig_name);
    }
    if !view.mobile.trim().is_empty() {
        if !view.signature_name.is_empty() {
            sx += space + 4.0;
            sx += b.line(sx, top_for(sig_text), "|", sig_text);
        }
        sx += space + 4.0;
        b.line(sx, top_for(sig_text), &format!("WhatsApp: {}", view.mobile.trim()), sig_text);
    }

    PageLayout {
        width: PAPER_WIDTH,
        height,
        background: C_PAPER,
        items: b.items,
        logo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LineItem;

    fn sample() -> InvoiceView {
        InvoiceView {
            number: "0001".into(),
            date: "16-Sep-2026".into(),
            client_name: "Acme Corporation".into(),
            to_location: "New York, USA".into(),
            to_email: "billing@acme.com".into(),
            from_name: "Jarif Hasan".into(),
            from_location: "Dhaka, Bangladesh".into(),
            from_email: "jarif@example.com".into(),
            items: vec![LineItem::new("Website Development", "https://example.com\nSecond line", 200.0)],
            symbol: "$".into(),
            show_advance: true,
            show_balance: true,
            advance: 50.0,
            note: String::new(),
            platforms: [("Payoneer".into(), "a@b.co".into()), ("Wise".into(), String::new())],
            bank_left: vec![("First Name".into(), "Jarif".into())],
            bank_right: vec![("Bank Name".into(), "Dhaka Bank".into())],
            signature_name: "Jarif Hasan".into(),
            mobile: "+880 1700-000000".into(),
        }
    }

    #[test]
    fn single_page_min_height() {
        let l = layout(&sample(), Logo::default_brand());
        assert_eq!(l.width, 1080.0);
        assert!(l.height >= PAPER_MIN_HEIGHT);
        assert!(!l.items.is_empty());
    }

    #[test]
    fn many_items_grow_the_page() {
        let mut v = sample();
        v.items = (0..30).map(|i| LineItem::new(&format!("Item {i}"), "", 10.0)).collect();
        let l = layout(&v, Logo::default_brand());
        assert!(l.height > 2000.0);
    }

    #[test]
    fn wraps_long_words() {
        let st = Style::new(Weight::Light, 18.0, C_TEXT).lh(24.0);
        let lines = wrap(&"x".repeat(200), st, 200.0, false);
        assert!(lines.len() > 1);
        let lines = wrap("a b c\nd", st, 500.0, true);
        assert_eq!(lines, vec!["a b c", "d"]);
    }

    #[test]
    fn fixed2_matches_js() {
        assert_eq!(fixed2(200.0), "200.00");
        assert_eq!(fixed2(-0.001), "0.00");
        assert_eq!(fixed2(1.005), "1.00");
        assert_eq!(fixed2(1500.0), "1,500.00");
        assert_eq!(fixed2(-1234567.5), "-1,234,567.50");
        assert_eq!(fixed2(999.999), "1,000.00");
    }
}
