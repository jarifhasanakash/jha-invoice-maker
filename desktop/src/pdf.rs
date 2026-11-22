//! Single-page vector PDF export with optional AES-256 edit protection.
//!
//! The page is exactly as tall as the invoice (no page breaks, no margins),
//! text is real selectable text with embedded (subset) fonts.
//!
//! Protection matches the original app: anyone can open, print and copy the
//! PDF without a password (empty user password); modifying, annotating, form
//! filling and page assembly require the owner password.

use std::collections::BTreeMap;
use std::sync::Arc;

use krilla::Document;
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point, Size, Transform};
use krilla::image::Image;
use krilla::metadata::Metadata;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, Stroke};
use krilla::text::{Font, GlyphId, KrillaGlyph};
use krilla_svg::{SurfaceExt, SvgSettings};

use crate::fonts;
use crate::layout::{PageLayout, Paint, Rgb};
use crate::logo::{LogoKind, RasterFormat};
use crate::render::{Cmd, rounded_rect};

/// CSS px → PDF pt (96 dpi → 72 dpi), same as Chromium's print-to-PDF.
const PX_TO_PT: f32 = 0.75;

fn color(c: Rgb) -> rgb::Color {
    rgb::Color::new(c[0], c[1], c[2])
}

fn fill(c: Rgb) -> Fill {
    Fill {
        paint: color(c).into(),
        opacity: NormalizedF32::ONE,
        rule: FillRule::NonZero,
    }
}

fn path(cmds: &[Cmd]) -> Option<krilla::geom::Path> {
    let mut pb = PathBuilder::new();
    for c in cmds {
        match *c {
            Cmd::Move(x, y) => pb.move_to(x, y),
            Cmd::Line(x, y) => pb.line_to(x, y),
            Cmd::Cubic(a, b, c, d, e, f) => pb.cubic_to(a, b, c, d, e, f),
            Cmd::Close => pb.close(),
        }
    }
    pb.finish()
}

/// Renders the page layout into PDF bytes (unencrypted).
pub fn render_pdf(page: &PageLayout, title: &str) -> Result<Vec<u8>, String> {
    let mut document = Document::new();
    document.set_metadata(
        Metadata::new()
            .title(title.to_string())
            .creator("JHA Invoice Maker".to_string())
            .producer("JHA Invoice Maker (Rust)".to_string()),
    );

    let book = fonts::book();
    let mut font_cache: BTreeMap<usize, Font> = BTreeMap::new();
    let mut get_font = |idx: usize| -> Option<Font> {
        if let Some(f) = font_cache.get(&idx) {
            return Some(f.clone());
        }
        let ff = &book.faces[idx];
        let font = Font::new(krilla::Data::from(ff.data), ff.index)?;
        font_cache.insert(idx, font.clone());
        Some(font)
    };

    let settings = PageSettings::from_wh(page.width * PX_TO_PT, page.height * PX_TO_PT)
        .ok_or("Invalid page size")?;
    let mut pdf_page = document.start_page_with(settings);
    let mut surface = pdf_page.surface();
    surface.push_transform(&Transform::from_scale(PX_TO_PT, PX_TO_PT));

    // Paper background
    if let Some(p) = path(&rounded_rect(0.0, 0.0, page.width, page.height, 0.0)) {
        surface.set_fill(Some(fill(page.background)));
        surface.set_stroke(None);
        surface.draw_path(&p);
    }

    for item in &page.items {
        match item {
            Paint::Rect {
                x,
                y,
                w,
                h,
                radius,
                fill: f,
                stroke,
            } => {
                let Some(p) = path(&rounded_rect(*x, *y, *w, *h, *radius)) else {
                    continue;
                };
                surface.set_fill(f.map(fill));
                surface.set_stroke(stroke.map(|c| Stroke {
                    paint: color(c).into(),
                    width: 1.0,
                    ..Stroke::default()
                }));
                surface.draw_path(&p);
            }
            Paint::Text {
                x,
                baseline,
                color: c,
                shaped,
            } => {
                surface.set_fill(Some(fill(*c)));
                surface.set_stroke(None);
                let size = shaped.size;
                for seg in &shaped.segments {
                    let Some(font) = get_font(seg.face) else { continue };
                    let glyphs: Vec<KrillaGlyph> = seg
                        .glyphs
                        .iter()
                        .map(|g| {
                            KrillaGlyph::new(
                                GlyphId::new(g.id as u32),
                                g.advance / size,
                                g.x_offset / size,
                                g.y_offset / size,
                                0.0,
                                g.range.clone(),
                                None,
                            )
                        })
                        .collect();
                    surface.draw_glyphs(
                        Point::from_xy(x + seg.x, *baseline),
                        &glyphs,
                        font,
                        &seg.text,
                        size,
                        false,
                    );
                }
            }
            Paint::Logo { x, y, w, h } => {
                let (lx, ly, lw, lh) = page.logo.fit(*x, *y, *w, *h);
                let Some(size) = Size::from_wh(lw, lh) else { continue };
                surface.push_transform(&Transform::from_translate(lx, ly));
                match &page.logo.kind {
                    LogoKind::Svg(tree) => {
                        surface.draw_svg(tree, size, SvgSettings::default());
                    }
                    LogoKind::Raster {
                        rgba,
                        width,
                        height,
                        encoded,
                        format,
                    } => {
                        let data = krilla::Data::from(Arc::clone(encoded));
                        let image = match format {
                            RasterFormat::Png => Image::from_png(data, true).ok(),
                            RasterFormat::Jpeg => Image::from_jpeg(data, true).ok(),
                            RasterFormat::Gif => Image::from_gif(data, true).ok(),
                            RasterFormat::Webp => Image::from_webp(data, true).ok(),
                            RasterFormat::Other => None,
                        }
                        .unwrap_or_else(|| Image::from_rgba8((**rgba).clone(), *width, *height));
                        surface.draw_image(image, size);
                    }
                }
                surface.pop();
            }
        }
    }

    surface.pop();
    surface.finish();
    pdf_page.finish();
    document
        .finish()
        .map_err(|e| format!("PDF generation failed: {e:?}"))
}

/// Applies AES-256 (PDF 2.0, V5/R6) owner-password protection.
pub fn protect(pdf: &[u8], owner_password: &str) -> Result<Vec<u8>, String> {
    use lopdf::encryption::crypt_filters::{Aes256CryptFilter, CryptFilter};
    use lopdf::{EncryptionState, EncryptionVersion, Object, Permissions, StringFormat};

    let mut doc = lopdf::Document::load_mem(pdf).map_err(|e| format!("PDF encryption failed: {e}"))?;

    if doc.trailer.get(b"ID").is_err() {
        let mut id = vec![0u8; 16];
        getrandom::fill(&mut id).map_err(|e| format!("PDF encryption failed: {e}"))?;
        doc.trailer.set(
            "ID",
            Object::Array(vec![
                Object::String(id.clone(), StringFormat::Hexadecimal),
                Object::String(id, StringFormat::Hexadecimal),
            ]),
        );
    }

    let mut key = [0u8; 32];
    getrandom::fill(&mut key).map_err(|e| format!("PDF encryption failed: {e}"))?;
    let filter: Arc<dyn CryptFilter> = Arc::new(Aes256CryptFilter);
    let permissions = Permissions::PRINTABLE
        | Permissions::PRINTABLE_IN_HIGH_QUALITY
        | Permissions::COPYABLE
        | Permissions::COPYABLE_FOR_ACCESSIBILITY;
    let state = EncryptionState::try_from(EncryptionVersion::V5 {
        encrypt_metadata: true,
        crypt_filters: BTreeMap::from([(b"StdCF".to_vec(), filter)]),
        file_encryption_key: &key,
        stream_filter: b"StdCF".to_vec(),
        string_filter: b"StdCF".to_vec(),
        owner_password,
        user_password: "",
        permissions,
    })
    .map_err(|e| format!("PDF encryption failed: {e}"))?;
    doc.encrypt(&state)
        .map_err(|e| format!("PDF encryption failed: {e}"))?;

    let mut out = Vec::new();
    doc.save_to(&mut out)
        .map_err(|e| format!("PDF encryption failed: {e}"))?;
    Ok(out)
}

/// Full export: render, then protect when a password is set.
pub fn export(page: &PageLayout, title: &str, password: &str) -> Result<Vec<u8>, String> {
    let bytes = render_pdf(page, title)?;
    let pwd = password.trim();
    if pwd.is_empty() {
        Ok(bytes)
    } else {
        protect(&bytes, pwd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{InvoiceView, layout};
    use crate::logo::Logo;
    use crate::model::LineItem;

    fn page() -> PageLayout {
        let view = InvoiceView {
            number: "0042".into(),
            date: "16-Sep-2026".into(),
            client_name: "Acme Corporation".into(),
            to_location: "New York, USA".into(),
            to_email: "billing@acme.com".into(),
            from_name: "Jarif Hasan".into(),
            from_location: "Dhaka, Bangladesh".into(),
            from_email: "jarif@example.com".into(),
            items: vec![LineItem::new("Website Development", "https://example.com", 200.0)],
            symbol: "৳".into(),
            platforms: [("Payoneer".into(), "a@b.co".into()), ("Wise".into(), String::new())],
            signature_name: "Jarif Hasan".into(),
            mobile: "+880".into(),
            ..Default::default()
        };
        layout(&view, Logo::default_brand())
    }

    #[test]
    fn renders_single_page_pdf() {
        let bytes = render_pdf(&page(), "Invoice - 0042").unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
        let doc = lopdf::Document::load_mem(&bytes).unwrap();
        assert_eq!(doc.get_pages().len(), 1);
        let text = doc.extract_text(&[1]).unwrap();
        assert!(text.contains("Acme"), "extracted: {text}");
    }

    #[test]
    fn protected_pdf_opens_without_password_but_is_encrypted() {
        let bytes = export(&page(), "Invoice - 0042", "s3cret").unwrap();
        let raw = lopdf::Document::load_mem_with_options(
            &bytes,
            lopdf::LoadOptions::with_password(""),
        )
        .unwrap();
        assert_eq!(raw.get_pages().len(), 1);
        let text = raw.extract_text(&[1]).unwrap();
        assert!(text.contains("Acme"), "extracted: {text}");
        // The encryption dictionary is present and uses AES-256 (V5/R6).
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/V 5") || s.contains("/V5"), "no V5 encryption dictionary");
        assert!(s.contains("/AESV3"));
    }
}

#[cfg(test)]
mod samples {
    use super::*;
    use crate::layout::{InvoiceView, layout};
    use crate::logo::Logo;
    use crate::model::LineItem;

    /// Writes sample PDFs to `$JHA_SAMPLE_DIR` for manual inspection.
    #[test]
    #[ignore]
    fn write_samples() {
        let dir = std::env::var("JHA_SAMPLE_DIR").expect("set JHA_SAMPLE_DIR");
        std::fs::write(
            format!("{dir}/demo-database.json"),
            serde_json::to_string_pretty(&crate::model::Database::demo()).unwrap(),
        )
        .unwrap();
        let mut view = InvoiceView {
            number: "0042".into(),
            date: "22-Nov-2026".into(),
            client_name: "Northwind Traders International Holdings Limited".into(),
            to_location: "Chittagong, Bangladesh".into(),
            to_email: "accounts@northwind.example".into(),
            from_name: "Sample Person".into(),
            from_location: "Dhaka, Bangladesh".into(),
            from_email: "sample@example.com".into(),
            items: vec![
                LineItem::new("Website Development", "https://example.com\nResponsive design + CMS setup", 1200.0),
                LineItem::new("Hosting (12 months)", "", 150.5),
                LineItem::new("Discount", "Loyal client", -100.0),
            ],
            symbol: "৳".into(),
            show_advance: true,
            show_balance: true,
            advance: 500.0,
            note: "Please complete your payment within 14 days, your cooperation is greatly appreciated.".into(),
            platforms: [("Payoneer".into(), "pay@example.com".into()), ("Wise".into(), "wise@example.com".into())],
            bank_left: vec![
                ("First Name".into(), "Sample".into()),
                ("Last Name".into(), "Person".into()),
                ("Address".into(), "Dhaka, Bangladesh".into()),
                ("A/C No".into(), "1234567890".into()),
            ],
            bank_right: vec![
                ("Bank Name".into(), "Example Bank".into()),
                ("Branch Name".into(), "Main Branch".into()),
                ("Branch Code".into(), "012".into()),
                ("SWIFT Code".into(), "EXAMBDDH".into()),
                ("Routing No".into(), "085260123".into()),
            ],
            signature_name: "Sample Person".into(),
            mobile: "+880 1700-000000".into(),
        };
        let page = layout(&view, Logo::default_brand());
        std::fs::write(format!("{dir}/sample-full.pdf"), render_pdf(&page, "Invoice - 0042").unwrap()).unwrap();
        std::fs::write(format!("{dir}/sample-protected.pdf"), export(&page, "Invoice - 0042", "s3cret").unwrap()).unwrap();
        view.items = (1..=25).map(|i| LineItem::new(&format!("Line item {i}"), "detail", 10.0)).collect();
        let page = layout(&view, Logo::default_brand());
        std::fs::write(format!("{dir}/sample-long.pdf"), render_pdf(&page, "Invoice - long").unwrap()).unwrap();
    }
}
