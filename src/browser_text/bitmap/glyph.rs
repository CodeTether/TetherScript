//! Five-by-seven fallback glyph painting.

use crate::browser::{RasterImage, Rgba};

use super::super::super::{fill_rect, glyph_rows};

pub(super) fn draw(
    image: &mut RasterImage,
    x: i64,
    y: i64,
    character: char,
    color: Rgba,
    scale: i64,
) {
    for (row_index, row) in glyph_rows(character).iter().enumerate() {
        for column in 0..5 {
            if row & (1 << (4 - column)) != 0 {
                fill_rect(
                    image,
                    x + column * scale,
                    y + row_index as i64 * scale,
                    scale,
                    scale,
                    color,
                );
            }
        }
    }
}
