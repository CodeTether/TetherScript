//! Cross-platform TrueType text rendering with an embedded OFL font.

#[path = "truetype/metrics.rs"]
mod metrics;
#[path = "truetype/paint.rs"]
mod paint;
#[cfg(test)]
#[path = "truetype/tests.rs"]
mod tests;

use std::sync::OnceLock;

use fontdue::{Font, FontSettings};

use crate::browser::{RasterImage, Rgba};

const FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/Inter.ttf");
static FONT: OnceLock<Font> = OnceLock::new();

pub(crate) fn draw(
    image: &mut RasterImage,
    x: i64,
    y: i64,
    text: &str,
    color: Rgba,
    scale: usize,
    size: i64,
) {
    if color.a == 0 {
        return;
    }
    let scale = scale.max(1);
    let size = size.max(1) as f32 * scale as f32;
    let origin_x = x.saturating_mul(scale as i64);
    let mut cursor_x = origin_x as f32;
    let mut cursor_y = y.saturating_mul(scale as i64);
    for character in text.chars() {
        if character == '\n' {
            cursor_x = origin_x as f32;
            cursor_y += size as i64;
            continue;
        }
        let (metrics, coverage) = font().rasterize(character, size);
        let x = cursor_x as i64 + i64::from(metrics.xmin);
        paint::glyph(image, x, cursor_y, &metrics, &coverage, color);
        cursor_x += metrics.advance_width;
    }
}

pub(crate) fn measure(text: &str, size: i64) -> (i64, i64) {
    metrics::text(font(), text, size.max(1) as f32)
}

fn font() -> &'static Font {
    FONT.get_or_init(|| {
        Font::from_bytes(FONT_BYTES, FontSettings::default())
            .expect("embedded Inter font must be valid")
    })
}
