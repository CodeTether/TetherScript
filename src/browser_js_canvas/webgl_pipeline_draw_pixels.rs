//! Scissor testing and color-masked fragment writes.

use super::super::super::super::surface::Surface;
use super::*;

pub(super) fn inside_scissor(x: usize, y: usize, scissor: Option<[i64; 4]>) -> bool {
    scissor.is_none_or(|rect| {
        x as i64 >= rect[0]
            && (x as i64) < rect[0] + rect[2]
            && y as i64 >= rect[1]
            && (y as i64) < rect[1] + rect[3]
    })
}

pub(super) fn write(
    surface: &mut Surface,
    x: usize,
    bottom_y: usize,
    call: &DrawCall,
    color: [u8; 4],
) {
    let top_y = surface.height as usize - 1 - bottom_y;
    let pixel = &mut surface.pixels[top_y * surface.width as usize + x];
    for (channel, target) in pixel.iter_mut().enumerate() {
        if call.channels[channel] {
            *target = color[channel];
        }
    }
}
