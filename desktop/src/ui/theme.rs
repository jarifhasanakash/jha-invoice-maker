//! Colours, fonts and egui style matching the original dark design system.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Shadow, Stroke,
    TextStyle, Vec2, style::ScrollStyle,
};

use crate::fonts::{self, Weight};

pub const BRAND: Color32 = Color32::from_rgb(0xea, 0x4e, 0x25);
pub const BRAND_HOVER: Color32 = Color32::from_rgb(0xd1, 0x3e, 0x18);
pub const BRAND_ACTIVE: Color32 = Color32::from_rgb(0xc3, 0x40, 0x1d);

pub const BG_APP: Color32 = Color32::from_rgb(0x12, 0x12, 0x12);
pub const BG_SIDEBAR: Color32 = Color32::from_rgb(0x1e, 0x1e, 0x1e);
pub const BG_SURFACE: Color32 = Color32::from_rgb(0x24, 0x24, 0x24);
pub const BG_SURFACE_ALT: Color32 = Color32::from_rgb(0x25, 0x25, 0x25);
pub const BG_SURFACE_HOVER: Color32 = Color32::from_rgb(0x28, 0x28, 0x28);
pub const BG_INPUT_HOVER: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
pub const BG_PREVIEW: Color32 = Color32::from_rgb(0x14, 0x14, 0x14);
pub const BG_TOPBAR: Color32 = Color32::from_rgb(0x18, 0x18, 0x18);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x1a);

pub const TEXT_WHITE: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xf0, 0xf0, 0xf0);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0xcc, 0xcc, 0xcc);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0xaa, 0xaa, 0xaa);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x88, 0x88, 0x88);
pub const TEXT_SUBTLE: Color32 = Color32::from_rgb(0x77, 0x77, 0x77);

pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const BORDER_MUTED: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
pub const BORDER_DEFAULT: Color32 = Color32::from_rgb(0x48, 0x48, 0x48);
pub const BORDER_LIGHT: Color32 = Color32::from_rgb(0x55, 0x55, 0x55);
pub const N720: Color32 = Color32::from_rgb(0x40, 0x40, 0x40);
pub const N780: Color32 = Color32::from_rgb(0x36, 0x36, 0x36);
pub const N840: Color32 = Color32::from_rgb(0x2f, 0x2f, 0x2f);

pub const SUCCESS: Color32 = Color32::from_rgb(0x4a, 0xde, 0x80);
pub const WARNING: Color32 = Color32::from_rgb(0xfa, 0xcc, 0x15);
pub const DANGER: Color32 = Color32::from_rgb(0xef, 0x44, 0x44);
pub const DANGER_TEXT: Color32 = Color32::from_rgb(0xf8, 0x71, 0x71);

pub fn alpha(c: Color32, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a.clamp(0.0, 1.0) * 255.0) as u8)
}

pub fn family(w: Weight) -> FontFamily {
    match w {
        Weight::Regular => FontFamily::Proportional,
        Weight::Light => FontFamily::Name("light".into()),
        Weight::Medium => FontFamily::Name("medium".into()),
        Weight::SemiBold => FontFamily::Name("semibold".into()),
        Weight::Bold => FontFamily::Name("bold".into()),
    }
}

pub fn font(size: f32, w: Weight) -> FontId {
    FontId::new(size, family(w))
}

pub fn status_color(status: &str) -> Color32 {
    match status.to_lowercase().as_str() {
        "paid" => SUCCESS,
        "pending" => WARNING,
        "overdue" => DANGER_TEXT,
        _ => Color32::from_rgb(0xa0, 0xa0, 0xa0),
    }
}

static CAPTION_FONT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// Font family holding the Windows caption-button glyphs (Segoe Fluent Icons / MDL2).
pub fn caption_family() -> FontFamily {
    FontFamily::Name("caption-icons".into())
}

/// True when the Windows icon font was found.
pub fn has_caption_font() -> bool {
    CAPTION_FONT.get().copied().unwrap_or(false)
}

pub fn install(ctx: &egui::Context) {
    // ---- Fonts: Lexend (all weights) + system fallbacks for ৳ etc. ----
    let mut defs = FontDefinitions::default();
    let book = fonts::book();
    let lexend = [
        ("lexend-light", Weight::Light, fonts::LEXEND_LIGHT),
        ("lexend-regular", Weight::Regular, fonts::LEXEND_REGULAR),
        ("lexend-medium", Weight::Medium, fonts::LEXEND_MEDIUM),
        ("lexend-semibold", Weight::SemiBold, fonts::LEXEND_SEMIBOLD),
        ("lexend-bold", Weight::Bold, fonts::LEXEND_BOLD),
    ];
    let mut fallbacks = Vec::new();
    for (i, face) in book.faces.iter().enumerate().skip(lexend.len()) {
        let name = format!("fallback-{i}");
        defs.font_data.insert(
            name.clone(),
            Arc::new(FontData::from_blob(Arc::new(face.data), face.index)),
        );
        fallbacks.push(name);
    }
    let egui_defaults = defs
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    for (name, weight, data) in lexend {
        defs.font_data
            .insert(name.to_string(), Arc::new(FontData::from_static(data)));
        let mut list = vec![name.to_string()];
        list.extend(fallbacks.iter().cloned());
        list.extend(egui_defaults.iter().cloned());
        defs.families.insert(family(weight), list);
    }

    // Windows caption-button glyphs (minimize / maximize / restore / close) come
    // from the system icon font, exactly like other Windows apps.
    let fonts_dir = std::env::var_os("WINDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "C:\\Windows".into())
        .join("Fonts");
    for file in ["SegoeIcons.ttf", "segmdl2.ttf"] {
        if let Ok(bytes) = std::fs::read(fonts_dir.join(file)) {
            defs.font_data.insert("caption-icons".into(), Arc::new(FontData::from_owned(bytes)));
            defs.families.insert(caption_family(), vec!["caption-icons".into()]);
            let _ = CAPTION_FONT.set(true);
            break;
        }
    }
    ctx.set_fonts(defs);

    // ---- Style ----
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Small, font(11.5, Weight::Regular)),
            (TextStyle::Body, font(13.5, Weight::Regular)),
            (TextStyle::Button, font(13.0, Weight::Medium)),
            (TextStyle::Heading, font(20.0, Weight::SemiBold)),
            (TextStyle::Monospace, FontId::monospace(13.0)),
        ]
        .into();
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(14.0, 8.0);
        style.spacing.interact_size.y = 36.0;
        style.spacing.scroll = ScrollStyle::thin();
        style.spacing.scroll.bar_width = 6.0;
        // Scrollbars float over content so they don't eat into the right padding.
        style.spacing.scroll.floating = true;
        style.spacing.scroll.floating_allocated_width = 0.0;
        style.spacing.menu_margin = 5.into();
        style.interaction.selectable_labels = false;

        let v = &mut style.visuals;
        v.dark_mode = true;
        v.panel_fill = BG_SIDEBAR;
        v.window_fill = BG_SURFACE;
        v.extreme_bg_color = BG_SURFACE;
        v.text_edit_bg_color = Some(BG_SURFACE);
        v.faint_bg_color = BG_SURFACE_HOVER;
        v.hyperlink_color = BRAND;
        v.window_stroke = Stroke::new(1.0, BORDER_SUBTLE);
        v.window_corner_radius = CornerRadius::same(10);
        v.menu_corner_radius = CornerRadius::same(8);
        v.window_shadow = Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color: Color32::from_black_alpha(150),
        };
        v.popup_shadow = Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color: Color32::from_black_alpha(150),
        };
        v.selection.bg_fill = alpha(BRAND, 0.35);
        v.selection.stroke = Stroke::new(1.0, BRAND);
        v.text_cursor.stroke = Stroke::new(1.5, TEXT_WHITE);

        let r = CornerRadius::same(8);
        let w = &mut v.widgets;
        w.noninteractive.bg_fill = BG_SURFACE;
        w.noninteractive.weak_bg_fill = BG_SURFACE;
        w.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
        w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
        w.noninteractive.corner_radius = r;

        w.inactive.bg_fill = BG_SURFACE;
        w.inactive.weak_bg_fill = BG_SURFACE;
        w.inactive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
        w.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
        w.inactive.corner_radius = r;

        w.hovered.bg_fill = BG_INPUT_HOVER;
        w.hovered.weak_bg_fill = BG_INPUT_HOVER;
        w.hovered.bg_stroke = Stroke::new(1.0, BORDER_LIGHT);
        w.hovered.fg_stroke = Stroke::new(1.0, TEXT_WHITE);
        w.hovered.corner_radius = r;
        w.hovered.expansion = 0.0;

        w.active.bg_fill = BG_SURFACE;
        w.active.weak_bg_fill = BG_SURFACE;
        w.active.bg_stroke = Stroke::new(1.0, BRAND);
        w.active.fg_stroke = Stroke::new(1.0, TEXT_WHITE);
        w.active.corner_radius = r;
        w.active.expansion = 0.0;

        w.open = w.active;
    });
}
