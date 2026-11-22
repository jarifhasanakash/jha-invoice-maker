//! Company logo handling. Logos are stored as data URLs (as in the original
//! app). SVG logos stay vector in the PDF; raster logos are embedded as-is.

use std::sync::Arc;

use base64::Engine;
use resvg::usvg;

/// The default JHA brand mark shown when no logo has been uploaded.
pub const DEFAULT_LOGO_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 400" width="400" height="400"><path fill="#ea4e25" d="M38.5,294.7l-12.8,3.2C8.8,268,0,234.3,0,200,0,115.3,52.9,42.8,127.4,13.6v29.5c-59.2,27.5-100.3,87.5-100.3,156.9,0,32.2,8.9,63.7,25.9,91l-14.6,3.7ZM200,0c-7.5,0-15,.4-22.4,1.2v27.3c7.4-1,14.9-1.5,22.4-1.5,95.3,0,172.9,77.6,172.9,172.9,0,6.3-.3,12.6-1,18.8l-236.4,59.6v35.2l228.1-57.5s0,0,0,0c-23.4,67.8-87.8,116.6-163.5,116.6-40.1,0-78.9-13.9-109.8-39.4l-31.5,7.9c36.2,36.2,86.2,58.6,141.3,58.6,93.5,0,172.3-64.6,194-151.5,3-12,4.9-24.3,5.6-36.6.2-3.9.4-7.9.4-11.9C400,89.7,310.3,0,200,0ZM169.6,197c0-2.2,1-4.3,2.6-5.9,1.6-1.6,3.7-2.4,6-2.4s4.4.9,6,2.4c1.6,1.6,2.5,3.7,2.6,5.9v50.9l34.2-8.6v-42.3c0-10.8-4.2-19.9-12.6-27.3-8.4-7.4-18.4-11.1-30.1-11.1-2.9,0-5.7-.1-8.5-.4V2.3c-11.6,1.8-23,4.6-34.1,8.4v250.1l34.2-8.6v-55.2ZM295.8,217.5v-7.2c-5.5,4.1-11.8,7.1-18.4,8.9-24.1,6.4-43.6-7.5-43.6-31.1s19.5-47.9,43.6-54.4c6-1.7,12.3-2,18.4-.9v-3.8l34.1-9.1v88.4l-34.2,9.2ZM286.1,174.4c0-1.7-.5-3.4-1.4-4.9-.9-1.5-2.3-2.6-3.9-3.3-1.6-.7-3.4-.9-5.1-.6-1.7.3-3.3,1.1-4.5,2.4-1.2,1.2-2.1,2.8-2.4,4.5-.4,1.7-.2,3.5.5,5.1.7,1.6,1.8,3,3.2,4,1.4,1,3.1,1.5,4.9,1.5,1.1,0,2.3-.2,3.3-.6,1.1-.4,2-1.1,2.8-1.9.8-.8,1.5-1.7,1.9-2.8.4-1,.7-2.2.7-3.3ZM118.3,141.4v-16.6l-34.1,9v16.6l34.1-9ZM84.2,326.7l34.1-8.6v-160l-34.1,8.5v124.8l-26.5,6.7-27.8,7c6.5,10.5,14,20.4,22.4,29.6l5-1.3,25.5-6.4,1.4-.3Z"/></svg>"##;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Other,
}

pub enum LogoKind {
    Svg(Arc<usvg::Tree>),
    Raster {
        /// Straight (non-premultiplied) RGBA8 pixels.
        rgba: Arc<Vec<u8>>,
        width: u32,
        height: u32,
        encoded: Arc<Vec<u8>>,
        format: RasterFormat,
    },
}

pub struct Logo {
    pub kind: LogoKind,
}

impl Logo {
    /// Intrinsic aspect ratio (width / height).
    pub fn aspect(&self) -> f32 {
        match &self.kind {
            LogoKind::Svg(tree) => {
                let s = tree.size();
                if s.height() > 0.0 { s.width() / s.height() } else { 1.0 }
            }
            LogoKind::Raster { width, height, .. } => {
                if *height > 0 { *width as f32 / *height as f32 } else { 1.0 }
            }
        }
    }

    /// Rect (x, y, w, h) for `object-fit: contain` inside the given box.
    pub fn fit(&self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
        let a = self.aspect();
        if a >= w / h {
            let nh = w / a;
            (x, y + (h - nh) / 2.0, w, nh)
        } else {
            let nw = h * a;
            (x + (w - nw) / 2.0, y, nw, h)
        }
    }

    pub fn default_brand() -> Arc<Logo> {
        static DEFAULT: std::sync::OnceLock<Arc<Logo>> = std::sync::OnceLock::new();
        DEFAULT
            .get_or_init(|| {
                Arc::new(Logo::from_svg(DEFAULT_LOGO_SVG.as_bytes()).expect("default logo parses"))
            })
            .clone()
    }

    pub fn from_svg(bytes: &[u8]) -> Result<Logo, String> {
        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();
        let tree = usvg::Tree::from_data(bytes, &opt).map_err(|e| format!("Invalid SVG: {e}"))?;
        Ok(Logo {
            kind: LogoKind::Svg(Arc::new(tree)),
        })
    }

    pub fn from_raster(bytes: Vec<u8>) -> Result<Logo, String> {
        let format = match image::guess_format(&bytes) {
            Ok(image::ImageFormat::Png) => RasterFormat::Png,
            Ok(image::ImageFormat::Jpeg) => RasterFormat::Jpeg,
            Ok(image::ImageFormat::Gif) => RasterFormat::Gif,
            Ok(image::ImageFormat::WebP) => RasterFormat::Webp,
            _ => RasterFormat::Other,
        };
        let img = image::load_from_memory(&bytes).map_err(|e| format!("Unsupported image: {e}"))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(Logo {
            kind: LogoKind::Raster {
                rgba: Arc::new(rgba.into_raw()),
                width,
                height,
                encoded: Arc::new(bytes),
                format,
            },
        })
    }

    /// Decodes raw file bytes (SVG or any supported raster format).
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Logo, String> {
        if looks_like_svg(&bytes) {
            Logo::from_svg(&bytes)
        } else {
            Logo::from_raster(bytes)
        }
    }

    /// Decodes a `data:` URL as stored in the database.
    pub fn from_data_url(url: &str) -> Result<Logo, String> {
        let (meta, payload) = url
            .strip_prefix("data:")
            .and_then(|rest| rest.split_once(','))
            .ok_or_else(|| "Logo is not a data URL".to_string())?;
        let bytes = if meta.ends_with(";base64") {
            base64::engine::general_purpose::STANDARD
                .decode(payload.trim())
                .map_err(|e| format!("Invalid logo data: {e}"))?
        } else {
            percent_decode(payload)
        };
        if meta.contains("svg") {
            Logo::from_svg(&bytes)
        } else {
            Logo::from_bytes(bytes)
        }
    }
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]).to_lowercase();
    head.contains("<svg") || (head.trim_start().starts_with("<?xml") && head.contains("svg"))
}

fn percent_decode(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

/// Builds a data URL for storage from raw file bytes.
pub fn to_data_url(bytes: &[u8], file_name: &str) -> String {
    let ext = file_name.rsplit('.').next().unwrap_or("").to_lowercase();
    let mime = if looks_like_svg(bytes) {
        "image/svg+xml"
    } else {
        match image::guess_format(bytes) {
            Ok(image::ImageFormat::Png) => "image/png",
            Ok(image::ImageFormat::Jpeg) => "image/jpeg",
            Ok(image::ImageFormat::Gif) => "image/gif",
            Ok(image::ImageFormat::WebP) => "image/webp",
            _ => match ext.as_str() {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                _ => "application/octet-stream",
            },
        }
    };
    format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}
