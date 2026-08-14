//! Deterministic dependency-free bitmap text fallback.

#[path = "bitmap/glyph.rs"]
mod glyph;
#[cfg(test)]
#[path = "bitmap/tests.rs"]
mod tests;

use crate::browser::{RasterImage, Rgba};

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
    let cell = size.max(1).saturating_mul(scale.max(1) as i64);
    let glyph_scale = (cell / 8).max(1);
    let origin = x.saturating_mul(scale.max(1) as i64);
    let mut cursor = origin;
    for character in text.chars() {
        if character == '\n' {
            cursor = origin;
            continue;
        }
        glyph::draw(
            image,
            cursor,
            y.saturating_mul(cell),
            character,
            color,
            glyph_scale,
        );
        cursor = cursor.saturating_add(cell);
    }
}

pub(crate) fn measure(text: &str, size: i64) -> (i64, i64) {
    (text.chars().count() as i64 * size.max(1), size.max(1))
}
