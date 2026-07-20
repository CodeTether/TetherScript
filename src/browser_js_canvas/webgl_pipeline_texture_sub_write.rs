//! Bounds-checked rectangular writes into allocated texture pixels.

use super::*;

pub(super) fn write(
    state: &mut WebGlState,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    pixels: &[[u8; 4]],
) {
    let Some(texture) = binding::get_mut(state) else {
        return;
    };
    let valid_x = x.checked_add(width).is_some_and(|end| end <= texture.width);
    let valid_y = y.checked_add(height).is_some_and(|end| end <= texture.height);
    if !valid_x || !valid_y {
        webgl_error::record(state, webgl_constants::INVALID_VALUE);
        return;
    }
    for row in 0..height {
        let source = row * width;
        let target = (y + row) * texture.width + x;
        texture.pixels[target..target + width]
            .copy_from_slice(&pixels[source..source + width]);
    }
}
