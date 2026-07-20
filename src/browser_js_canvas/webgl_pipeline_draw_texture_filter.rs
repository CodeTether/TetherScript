//! Minification versus magnification filter selection for one triangle.

use super::*;

pub(super) fn value(
    texture: &texture_state::Texture,
    vertices: &[Vertex],
    points: [[f64; 2]; 3],
) -> u32 {
    let mut ratio = 0.0_f64;
    for (left, right) in [(0, 1), (1, 2), (2, 0)] {
        let Some(a) = vertices[left].1 else {
            return texture.mag_filter;
        };
        let Some(b) = vertices[right].1 else {
            return texture.mag_filter;
        };
        let texels = ((a[0] - b[0]) * texture.width as f64)
            .hypot((a[1] - b[1]) * texture.height as f64);
        let pixels = (points[left][0] - points[right][0])
            .hypot(points[left][1] - points[right][1]);
        if pixels > 0.0 {
            ratio = ratio.max(texels / pixels);
        }
    }
    if ratio > 1.0 {
        texture.min_filter
    } else {
        texture.mag_filter
    }
}
