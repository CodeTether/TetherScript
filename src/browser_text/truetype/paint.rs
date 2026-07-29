//! Alpha-blended TrueType glyph painting.

use crate::browser::{RasterImage, Rgba};

pub(super) fn glyph(
    image: &mut RasterImage,
    x: i64,
    y: i64,
    metrics: &fontdue::Metrics,
    coverage: &[u8],
    color: Rgba,
) {
    for (index, coverage) in coverage.iter().copied().enumerate() {
        let alpha = (u16::from(coverage) * u16::from(color.a) / 255) as u8;
        let px = x + (index % metrics.width) as i64;
        let py = y + (index / metrics.width) as i64;
        image.set_pixel(px, py, Rgba { a: alpha, ..color });
    }
}
