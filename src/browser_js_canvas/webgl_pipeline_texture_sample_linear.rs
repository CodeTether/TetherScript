//! Bilinear filtering across four wrapped or clamped texels.

use super::*;

pub(super) fn color(texture: &texture_state::Texture, uv: [f64; 2]) -> [u8; 4] {
    let (x0, x1, tx) = sample_coordinate::linear(uv[0], texture.width, texture.wrap_s);
    let (y0, y1, ty) = sample_coordinate::linear(uv[1], texture.height, texture.wrap_t);
    let rows = [
        [pixel(texture, x0, y0), pixel(texture, x1, y0)],
        [pixel(texture, x0, y1), pixel(texture, x1, y1)],
    ];
    [0, 1, 2, 3].map(|channel| {
        let lower = mix(rows[0][0][channel] as f64, rows[0][1][channel] as f64, tx);
        let upper = mix(rows[1][0][channel] as f64, rows[1][1][channel] as f64, tx);
        mix(lower, upper, ty).round().clamp(0.0, 255.0) as u8
    })
}

fn pixel(texture: &texture_state::Texture, x: usize, y: usize) -> [u8; 4] {
    texture.pixels[y * texture.width + x]
}

fn mix(left: f64, right: f64, amount: f64) -> f64 {
    left * (1.0 - amount) + right * amount
}
