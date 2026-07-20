//! Complete level-zero texture sampling with nearest or linear filtering.

use super::*;

pub(super) fn color(texture: &texture_state::Texture, uv: [f64; 2], filter: u32) -> [u8; 4] {
    if texture.width == 0 || texture.height == 0 || texture.pixels.is_empty() {
        return [0, 0, 0, 255];
    }
    if filter == constants::LINEAR {
        return sample_linear::color(texture, uv);
    }
    let x = sample_coordinate::nearest(uv[0], texture.width, texture.wrap_s);
    let y = sample_coordinate::nearest(uv[1], texture.height, texture.wrap_t);
    texture.pixels[y * texture.width + x]
}
