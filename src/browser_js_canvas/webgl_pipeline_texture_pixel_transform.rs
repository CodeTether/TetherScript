//! Pixel-row orientation and alpha-premultiplication transforms.

use super::*;

pub(super) fn apply(
    items: &[JsValue],
    width: usize,
    height: usize,
    bindings: &texture_state::Bindings,
) -> Vec<[u8; 4]> {
    let mut output = Vec::with_capacity(width * height);
    for y in 0..height {
        let source_y = if bindings.flip_y { height - 1 - y } else { y };
        for x in 0..width {
            let start = (source_y * width + x) * 4;
            let mut pixel = [0, 1, 2, 3].map(|offset| byte(&items[start + offset]));
            if bindings.premultiply_alpha {
                let alpha = pixel[3];
                pixel[..3]
                    .iter_mut()
                    .for_each(|value| *value = scale(*value, alpha));
            }
            output.push(pixel);
        }
    }
    output
}

fn byte(value: &JsValue) -> u8 {
    value.display().parse::<f64>().unwrap_or(0.0).clamp(0.0, 255.0) as u8
}

fn scale(value: u8, alpha: u8) -> u8 {
    ((value as u16 * alpha as u16 + 127) / 255) as u8
}