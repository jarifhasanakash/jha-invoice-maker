//! Rasteriser for the on-screen invoice preview.
//!
//! Glyphs are filled from the real font outlines with the exact positions the
//! PDF writer uses, so what you see is what gets exported.

use std::collections::HashMap;

use resvg::tiny_skia::{
    self, FillRule, FilterQuality, Paint as SkPaint, PathBuilder, Pixmap, PixmapPaint, Stroke,
    Transform,
};
use rustybuzz::ttf_parser;

use crate::fonts;
use crate::layout::{PageLayout, Paint, Rgb};
use crate::logo::LogoKind;

/// Path commands for a rounded rectangle (shared with the PDF writer).
pub enum Cmd {
    Move(f32, f32),
    Line(f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}

pub fn rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<Cmd> {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    if r <= 0.0 {
        return vec![
            Cmd::Move(x, y),
            Cmd::Line(x + w, y),
            Cmd::Line(x + w, y + h),
            Cmd::Line(x, y + h),
            Cmd::Close,
        ];
    }
    let k = r * 0.552_284_8;
    vec![
        Cmd::Move(x + r, y),
        Cmd::Line(x + w - r, y),
        Cmd::Cubic(x + w - r + k, y, x + w, y + r - k, x + w, y + r),
        Cmd::Line(x + w, y + h - r),
        Cmd::Cubic(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h),
        Cmd::Line(x + r, y + h),
        Cmd::Cubic(x + r - k, y + h, x, y + h - r + k, x, y + h - r),
        Cmd::Line(x, y + r),
        Cmd::Cubic(x, y + r - k, x + r - k, y, x + r, y),
        Cmd::Close,
    ]
}

fn sk_path(cmds: &[Cmd]) -> Option<tiny_skia::Path> {
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

fn paint_rgb(c: Rgb) -> SkPaint<'static> {
    let mut p = SkPaint::default();
    p.set_color_rgba8(c[0], c[1], c[2], 255);
    p.anti_alias = true;
    p
}

struct OutlineSink(PathBuilder);

impl ttf_parser::OutlineBuilder for OutlineSink {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.0.close();
    }
}

/// Keeps glyph outlines between renders.
#[derive(Default)]
pub struct Rasterizer {
    glyphs: HashMap<(usize, u16), Option<tiny_skia::Path>>,
}

impl Rasterizer {
    fn glyph(&mut self, face: usize, id: u16) -> Option<&tiny_skia::Path> {
        self.glyphs
            .entry((face, id))
            .or_insert_with(|| {
                let ff = &fonts::book().faces[face];
                let mut sink = OutlineSink(PathBuilder::new());
                ff.face.outline_glyph(ttf_parser::GlyphId(id), &mut sink)?;
                sink.0.finish()
            })
            .as_ref()
    }

    /// Renders the page at `scale` device pixels per CSS pixel.
    /// Returns premultiplied RGBA pixels and the pixel size.
    pub fn render(&mut self, page: &PageLayout, scale: f32) -> (Vec<u8>, [usize; 2]) {
        let w = (page.width * scale).round().max(1.0) as u32;
        let h = (page.height * scale).round().max(1.0) as u32;
        let mut pixmap = Pixmap::new(w, h).expect("pixmap size is valid");
        let bg = page.background;
        pixmap.fill(tiny_skia::Color::from_rgba8(bg[0], bg[1], bg[2], 255));
        let base = Transform::from_scale(scale, scale);

        for item in &page.items {
            match item {
                Paint::Rect {
                    x,
                    y,
                    w,
                    h,
                    radius,
                    fill,
                    stroke,
                } => {
                    let Some(path) = sk_path(&rounded_rect(*x, *y, *w, *h, *radius)) else {
                        continue;
                    };
                    if let Some(c) = fill {
                        pixmap.fill_path(&path, &paint_rgb(*c), FillRule::Winding, base, None);
                    }
                    if let Some(c) = stroke {
                        let st = Stroke {
                            width: 1.0,
                            ..Stroke::default()
                        };
                        pixmap.stroke_path(&path, &paint_rgb(*c), &st, base, None);
                    }
                }
                Paint::Text {
                    x,
                    baseline,
                    color,
                    shaped,
                } => {
                    let paint = paint_rgb(*color);
                    for seg in &shaped.segments {
                        let upem = fonts::book().faces[seg.face].units_per_em;
                        let k = shaped.size / upem;
                        for g in &seg.glyphs {
                            let gx = x + seg.x + g.x + g.x_offset;
                            let gy = baseline - g.y_offset;
                            if let Some(path) = self.glyph(seg.face, g.id) {
                                let t = base.pre_translate(gx, gy).pre_scale(k, -k);
                                pixmap.fill_path(path, &paint, FillRule::Winding, t, None);
                            }
                        }
                    }
                }
                Paint::Logo { x, y, w, h } => {
                    let (lx, ly, lw, lh) = page.logo.fit(*x, *y, *w, *h);
                    match &page.logo.kind {
                        LogoKind::Svg(tree) => {
                            let s = tree.size();
                            let t = base
                                .pre_translate(lx, ly)
                                .pre_scale(lw / s.width(), lh / s.height());
                            resvg::render(tree, t, &mut pixmap.as_mut());
                        }
                        LogoKind::Raster {
                            rgba,
                            width,
                            height,
                            ..
                        } => {
                            if let Some(img) = premultiplied_pixmap(rgba, *width, *height) {
                                let t = base
                                    .pre_translate(lx, ly)
                                    .pre_scale(lw / *width as f32, lh / *height as f32);
                                let pp = PixmapPaint {
                                    quality: FilterQuality::Bicubic,
                                    ..PixmapPaint::default()
                                };
                                pixmap.draw_pixmap(0, 0, img.as_ref(), &pp, t, None);
                            }
                        }
                    }
                }
            }
        }

        (pixmap.take(), [w as usize, h as usize])
    }
}

pub fn premultiplied_pixmap(rgba: &[u8], w: u32, h: u32) -> Option<Pixmap> {
    let mut data = Vec::with_capacity(rgba.len());
    for px in rgba.chunks_exact(4) {
        let a = px[3] as u16;
        data.push(((px[0] as u16 * a + 127) / 255) as u8);
        data.push(((px[1] as u16 * a + 127) / 255) as u8);
        data.push(((px[2] as u16 * a + 127) / 255) as u8);
        data.push(px[3]);
    }
    Pixmap::from_vec(data, tiny_skia::IntSize::from_wh(w, h)?)
}

/// Rasterises an SVG (used for UI icons and the logo thumbnail).
pub fn svg_to_rgba(svg: &[u8], size: u32, color: Option<[u8; 3]>) -> Option<(Vec<u8>, [usize; 2])> {
    let text = String::from_utf8_lossy(svg);
    let text = match color {
        Some(c) => text.replace(
            "currentColor",
            &format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]),
        ),
        None => text.into_owned(),
    };
    let tree = resvg::usvg::Tree::from_data(text.as_bytes(), &resvg::usvg::Options::default()).ok()?;
    let s = tree.size();
    let scale = size as f32 / s.width().max(s.height());
    let w = (s.width() * scale).round().max(1.0) as u32;
    let h = (s.height() * scale).round().max(1.0) as u32;
    let mut pixmap = Pixmap::new(w, h)?;
    resvg::render(&tree, Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    Some((pixmap.take(), [w as usize, h as usize]))
}
