//! JHA Invoice Maker — PDF engine for the web version.
//!
//! The same Rust code as the desktop app (layout, fonts, vector PDF writer and
//! AES-256 edit protection), compiled to WebAssembly. It has no JavaScript
//! bindings generator: `web/pdf-engine.js` talks to it through a few plain
//! exported functions and linear memory.
//!
//! Input (UTF-8 JSON):
//! `{ "profile": {...}, "invoice": {...}, "password": "..." }`
//! using the same schema 2.1 objects the app stores. Output: PDF bytes.

#![allow(dead_code)]

#[path = "../../desktop/src/currency.rs"]
mod currency;
#[path = "../../desktop/src/dates.rs"]
mod dates;
#[path = "../../desktop/src/fonts.rs"]
mod fonts;
#[path = "../../desktop/src/layout.rs"]
mod layout;
#[path = "../../desktop/src/logo.rs"]
mod logo;
#[path = "../../desktop/src/model.rs"]
mod model;
#[path = "../../desktop/src/pdf.rs"]
mod pdf;
#[path = "../../desktop/src/render.rs"]
mod render;
#[path = "../../desktop/src/view.rs"]
mod view;

use std::cell::RefCell;
use std::sync::Arc;

use serde::Deserialize;

use crate::logo::Logo;
use crate::model::{Invoice, Profile};

#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    profile: Profile,
    invoice: Invoice,
    #[serde(default)]
    password: Option<String>,
}

thread_local! {
    /// Last decoded company logo, keyed by its data URL (decoding is slow).
    static LOGO: RefCell<Option<(String, Arc<Logo>)>> = const { RefCell::new(None) };
    /// Glyph outline cache for the raster preview.
    static RASTER: RefCell<render::Rasterizer> = RefCell::new(render::Rasterizer::default());
}

fn logo_for(profile: &Profile) -> Arc<Logo> {
    let url = profile.company_logo.as_deref().unwrap_or("").trim().to_string();
    if url.is_empty() {
        return Logo::default_brand();
    }
    LOGO.with(|cell| {
        if let Some((key, logo)) = cell.borrow().as_ref() {
            if *key == url {
                return logo.clone();
            }
        }
        let logo = Logo::from_data_url(&url).map(Arc::new).unwrap_or_else(|_| Logo::default_brand());
        *cell.borrow_mut() = Some((url, logo.clone()));
        logo
    })
}

/// Builds the invoice PDF (protected when a password is given).
pub fn export_pdf_json(json: &str) -> Result<Vec<u8>, String> {
    let req: Request = serde_json::from_str(json).map_err(|e| format!("Invalid invoice data: {e}"))?;
    let logo = logo_for(&req.profile);
    let view = view::view_for(&req.profile, &req.invoice);
    let page = layout::layout(&view, logo);
    let password = req
        .password
        .unwrap_or_else(|| req.profile.pdf_password.clone());
    pdf::export(&page, &format!("Invoice - {}", view.number), &password)
}

/* ------------------------------------------------------------------------- */
/* WebAssembly interface                                                      */
/* ------------------------------------------------------------------------- */

thread_local! {
    static RESULT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Allocates `len` bytes for the caller to write the request into.
#[unsafe(no_mangle)]
pub extern "C" fn jha_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len.max(1));
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Frees a buffer returned by `jha_alloc`.
///
/// # Safety
/// `ptr`/`cap` must come from `jha_alloc(cap)`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jha_free(ptr: *mut u8, cap: usize) {
    if !ptr.is_null() {
        drop(unsafe { Vec::from_raw_parts(ptr, 0, cap.max(1)) });
    }
}

/// Renders the PDF for the JSON request at `ptr..ptr+len`.
/// Returns 0 on success (result = PDF bytes) or 1 on error (result = UTF-8 message).
///
/// # Safety
/// `ptr..ptr+len` must be a readable buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jha_export_pdf(ptr: *const u8, len: usize) -> u32 {
    let input = unsafe { std::slice::from_raw_parts(ptr, len) };
    let (status, bytes) = match std::str::from_utf8(input) {
        Ok(json) => match export_pdf_json(json) {
            Ok(pdf) => (0, pdf),
            Err(e) => (1, e.into_bytes()),
        },
        Err(_) => (1, b"Request is not valid UTF-8".to_vec()),
    };
    RESULT.with(|r| *r.borrow_mut() = bytes);
    status
}

/// Renders the invoice preview (same renderer as the desktop app) at
/// `scale_milli / 1000` device pixels per CSS pixel.
/// Returns 0 on success (result = width u32 LE, height u32 LE, RGBA pixels)
/// or 1 on error (result = UTF-8 message).
///
/// # Safety
/// `ptr..ptr+len` must be a readable buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jha_render_preview(ptr: *const u8, len: usize, scale_milli: u32) -> u32 {
    let input = unsafe { std::slice::from_raw_parts(ptr, len) };
    let scale = (scale_milli as f32 / 1000.0).clamp(0.05, 4.0);
    let rendered = std::str::from_utf8(input)
        .map_err(|_| "Request is not valid UTF-8".to_string())
        .and_then(|json| serde_json::from_str::<Request>(json).map_err(|e| format!("Invalid invoice data: {e}")))
        .map(|req| {
            let view = view::view_for(&req.profile, &req.invoice);
            let page = layout::layout(&view, logo_for(&req.profile));
            RASTER.with(|r| r.borrow_mut().render(&page, scale))
        });
    let (status, bytes) = match rendered {
        Ok((pixels, [w, h])) => {
            let mut out = Vec::with_capacity(8 + pixels.len());
            out.extend_from_slice(&(w as u32).to_le_bytes());
            out.extend_from_slice(&(h as u32).to_le_bytes());
            out.extend_from_slice(&pixels);
            (0, out)
        }
        Err(e) => (1, e.into_bytes()),
    };
    RESULT.with(|r| *r.borrow_mut() = bytes);
    status
}

#[unsafe(no_mangle)]
pub extern "C" fn jha_result_ptr() -> *const u8 {
    RESULT.with(|r| r.borrow().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn jha_result_len() -> usize {
    RESULT.with(|r| r.borrow().len())
}

/// Frees the last result.
#[unsafe(no_mangle)]
pub extern "C" fn jha_result_clear() {
    RESULT.with(|r| *r.borrow_mut() = Vec::new());
}

/* Entropy for AES keys / document IDs: provided by the JS loader. */
#[cfg(target_arch = "wasm32")]
mod entropy {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        fn jha_random_fill(ptr: *mut u8, len: usize);
    }

    #[unsafe(no_mangle)]
    unsafe extern "Rust" fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), getrandom::Error> {
        unsafe { jha_random_fill(dest, len) };
        Ok(())
    }
}
