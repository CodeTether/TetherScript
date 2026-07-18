//! Scissored, channel-masked WebGL color-buffer clearing.

use super::super::surface::Surface;

pub(super) fn apply(
    surface: &mut Surface,
    color: [u8; 4],
    channels: [bool; 4],
    scissor: Option<[i64; 4]>,
) {
    if surface.pixels.is_empty() {
        return;
    }
    let rect = scissor.map_or(
        (0, 0, surface.width as i64, surface.height as i64),
        |rect| {
            (
                rect[0],
                (surface.height as i64).saturating_sub(rect[1].saturating_add(rect[3])),
                rect[2],
                rect[3],
            )
        },
    );
    let Some((x0, y0, x1, y1)) = super::super::geometry::clip(rect, surface.width, surface.height)
    else {
        return;
    };
    for y in y0..y1 {
        for x in x0..x1 {
            let pixel = &mut surface.pixels[y * surface.width as usize + x];
            for channel in 0..4 {
                if channels[channel] {
                    pixel[channel] = color[channel];
                }
            }
        }
    }
}
