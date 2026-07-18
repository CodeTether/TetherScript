//! Floating-point vertex attribute format validation.

use super::*;

pub(super) fn parse(state: &mut WebGlState, args: &[JsValue]) -> Option<Format> {
    let size = webgl_values::i64_value(args.get(1));
    let kind = webgl_values::u32_value(args.get(2));
    let stride = webgl_values::i64_value(args.get(4));
    let offset = webgl_values::i64_value(args.get(5));
    if !(1..=4).contains(&size) || !(0..=255).contains(&stride) || offset < 0 {
        webgl_error::record(state, webgl_constants::INVALID_VALUE);
        return None;
    }
    if kind != constants::FLOAT {
        webgl_error::record(state, webgl_constants::INVALID_ENUM);
        return None;
    }
    if stride % 4 != 0 || offset % 4 != 0 {
        webgl_error::record(state, webgl_constants::INVALID_OPERATION);
        return None;
    }
    Some(Format {
        size: size as usize,
        stride: if stride == 0 {
            size as usize * 4
        } else {
            stride as usize
        },
        offset: offset as usize,
    })
}
