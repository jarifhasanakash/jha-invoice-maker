//! Font loading and text shaping shared by the on-screen preview and the PDF
//! writer, so both produce exactly the same glyph positions.
//!
//! Lexend (bundled) is the primary font. The Taka sign ৳, which Lexend lacks,
//! comes from a bundled Noto Sans Bengali subset; other missing characters fall
//! back to Windows system fonts when available (desktop only).

use std::sync::OnceLock;

use rustybuzz::{Face, UnicodeBuffer};

pub static LEXEND_LIGHT: &[u8] = include_bytes!("../assets/fonts/Lexend-Light.ttf");
pub static LEXEND_REGULAR: &[u8] = include_bytes!("../assets/fonts/Lexend-Regular.ttf");
pub static LEXEND_MEDIUM: &[u8] = include_bytes!("../assets/fonts/Lexend-Medium.ttf");
pub static LEXEND_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Lexend-SemiBold.ttf");
pub static LEXEND_BOLD: &[u8] = include_bytes!("../assets/fonts/Lexend-Bold.ttf");
/// Noto Sans Bengali (SIL OFL 1.1), subset to the Taka sign.
pub static NOTO_TAKA: &[u8] = include_bytes!("../assets/fonts/NotoSansBengali-Taka.ttf");

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Weight {
    Light,
    Regular,
    Medium,
    SemiBold,
    Bold,
}

impl Weight {
    fn index(self) -> usize {
        match self {
            Weight::Light => 0,
            Weight::Regular => 1,
            Weight::Medium => 2,
            Weight::SemiBold => 3,
            Weight::Bold => 4,
        }
    }
}

pub struct FontFace {
    pub data: &'static [u8],
    pub index: u32,
    pub face: Face<'static>,
    pub units_per_em: f32,
}

pub struct FontBook {
    pub faces: Vec<FontFace>,
    /// Primary face per weight (indices into `faces`).
    primary: [usize; 5],
    /// System fallback faces, tried in order.
    fallbacks: Vec<usize>,
}

/// Ascent and descent (as a fraction of the font size) used for line boxes.
/// Lexend: typo ascender 1000, descender -250 at 1000 upm, line gap 0.
pub const ASCENT: f32 = 1.0;
pub const DESCENT: f32 = 0.25;
/// CSS `line-height: normal` for Lexend.
pub const NORMAL_LINE_HEIGHT: f32 = ASCENT + DESCENT;

#[derive(Clone, Debug)]
pub struct GlyphPos {
    pub id: u16,
    /// Pen position relative to the segment start (px), excluding `x_offset`.
    pub x: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub advance: f32,
    /// Byte range in the segment text covered by this glyph's cluster.
    pub range: std::ops::Range<usize>,
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub face: usize,
    pub text: String,
    /// Start of this segment relative to the run start (px).
    pub x: f32,
    pub glyphs: Vec<GlyphPos>,
}

#[derive(Clone, Debug, Default)]
pub struct ShapedText {
    pub segments: Vec<Segment>,
    pub width: f32,
    pub size: f32,
}

static BOOK: OnceLock<FontBook> = OnceLock::new();

pub fn book() -> &'static FontBook {
    BOOK.get_or_init(FontBook::load)
}

fn make_face(data: &'static [u8], index: u32) -> Option<FontFace> {
    let face = Face::from_slice(data, index)?;
    let units_per_em = face.units_per_em() as f32;
    Some(FontFace {
        data,
        index,
        face,
        units_per_em,
    })
}

impl FontBook {
    fn load() -> Self {
        let mut faces = Vec::new();
        for data in [LEXEND_LIGHT, LEXEND_REGULAR, LEXEND_MEDIUM, LEXEND_SEMIBOLD, LEXEND_BOLD] {
            faces.push(make_face(data, 0).expect("bundled Lexend font is valid"));
        }
        let primary = [0, 1, 2, 3, 4];

        let mut fallbacks = Vec::new();
        if let Some(face) = make_face(NOTO_TAKA, 0) {
            fallbacks.push(faces.len());
            faces.push(face);
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::load_system_fallbacks(&mut faces, &mut fallbacks);

        Self {
            faces,
            primary,
            fallbacks,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_system_fallbacks(faces: &mut Vec<FontFace>, fallbacks: &mut Vec<usize>) {
        let fonts_dir = std::env::var_os("WINDIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "C:\\Windows".into())
            .join("Fonts");
        for (file, index) in [
            ("Nirmala.ttc", 0u32),
            ("Nirmala.ttf", 0),
            ("segoeui.ttf", 0),
            ("seguisym.ttf", 0),
        ] {
            let path = fonts_dir.join(file);
            if let Ok(bytes) = std::fs::read(&path) {
                // Fonts live for the whole program; leaking gives us 'static faces.
                let data: &'static [u8] = Box::leak(bytes.into_boxed_slice());
                if let Some(face) = make_face(data, index) {
                    fallbacks.push(faces.len());
                    faces.push(face);
                }
            }
        }
    }

    pub fn primary(&self, weight: Weight) -> usize {
        self.primary[weight.index()]
    }

    fn face_for_char(&self, primary: usize, ch: char) -> usize {
        if self.faces[primary].face.glyph_index(ch).is_some() {
            return primary;
        }
        self.fallbacks
            .iter()
            .copied()
            .find(|&f| self.faces[f].face.glyph_index(ch).is_some())
            .unwrap_or(primary)
    }

    /// Shapes a single line of text (no line breaking) at `size` px.
    pub fn shape(&self, text: &str, weight: Weight, size: f32) -> ShapedText {
        let primary = self.primary(weight);

        // Split into runs that use the same face (font fallback).
        let mut runs: Vec<(usize, String)> = Vec::new();
        for ch in text.chars() {
            let face = if ch.is_whitespace() || ch.is_control() {
                runs.last().map(|r| r.0).unwrap_or(primary)
            } else {
                self.face_for_char(primary, ch)
            };
            match runs.last_mut() {
                Some((f, s)) if *f == face => s.push(ch),
                _ => runs.push((face, ch.to_string())),
            }
        }

        let mut segments = Vec::with_capacity(runs.len());
        let mut pen = 0.0;
        for (face_idx, run_text) in runs {
            let ff = &self.faces[face_idx];
            let scale = size / ff.units_per_em;
            let mut buffer = UnicodeBuffer::new();
            buffer.push_str(&run_text);
            buffer.guess_segment_properties();
            let output = rustybuzz::shape(&ff.face, &[], buffer);
            let infos = output.glyph_infos();
            let positions = output.glyph_positions();

            let mut glyphs = Vec::with_capacity(infos.len());
            let start = pen;
            let mut x = 0.0;
            for (i, (info, pos)) in infos.iter().zip(positions).enumerate() {
                let cluster = info.cluster as usize;
                let end = infos[i + 1..]
                    .iter()
                    .map(|g| g.cluster as usize)
                    .find(|&c| c > cluster)
                    .unwrap_or(run_text.len());
                let advance = pos.x_advance as f32 * scale;
                glyphs.push(GlyphPos {
                    id: info.glyph_id as u16,
                    x,
                    x_offset: pos.x_offset as f32 * scale,
                    y_offset: pos.y_offset as f32 * scale,
                    advance,
                    range: cluster..end,
                });
                x += advance;
            }
            pen += x;
            segments.push(Segment {
                face: face_idx,
                text: run_text,
                x: start,
                glyphs,
            });
        }

        ShapedText {
            segments,
            width: pen,
            size,
        }
    }

    pub fn width(&self, text: &str, weight: Weight, size: f32) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        self.shape(text, weight, size).width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_text_and_falls_back() {
        let b = book();
        let s = b.shape("Invoice", Weight::Regular, 36.0);
        assert!(s.width > 100.0 && s.width < 200.0, "width {}", s.width);
        assert_eq!(s.segments.len(), 1);
        // The Taka sign is not in Lexend; with a Windows fallback it gets its own segment.
        let t = b.shape("৳100", Weight::Regular, 18.0);
        assert!(t.width > 0.0);
    }
}
