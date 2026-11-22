//! JHA Invoice Maker — native Rust desktop application.
//!
//! Single portable executable: egui UI, vector PDF export (krilla) with
//! optional AES-256 edit protection (lopdf), and a local JSON database.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod currency;
mod dates;
mod fonts;
mod layout;
mod logo;
mod model;
mod pdf;
mod render;
mod storage;
mod ui;
mod view;

use std::sync::Arc;

use eframe::egui;

fn app_icon() -> Option<Arc<egui::IconData>> {
    let img = image::load_from_memory(include_bytes!("../assets/app-icon.png")).ok()?;
    let rgba = img.to_rgba8();
    Some(Arc::new(egui::IconData {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    }))
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("JHA Invoice Maker")
        .with_app_id("com.jarifhasan.invoicemaker")
        .with_inner_size([1380.0, 920.0])
        .with_min_inner_size([800.0, 600.0]);
    // Design-review mode (JHA_REVIEW=1): open without taking keyboard focus.
    if std::env::var_os("JHA_REVIEW").is_some() {
        viewport = viewport.with_active(false);
    }
    if let Some(icon) = app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "JHA Invoice Maker",
        options,
        Box::new(|cc| Ok(Box::new(ui::App::new(cc)))),
    )
}
