//! Scalar mode and range validation shared by WebGL draw entry points.

use super::*;

pub(super) fn mode(state: &mut WebGlState, value: Option<&JsValue>) -> Option<u32> {
    let mode = webgl_values::u32_value(value);
    if mode == constants::TRIANGLES {
        return Some(mode);
    }
    webgl_error::record(state, webgl_constants::INVALID_ENUM);
    None
}

pub(super) fn non_negative(state: &mut WebGlState, value: Option<&JsValue>) -> Option<usize> {
    let value = webgl_values::i64_value(value);
    if value >= 0 {
        return Some(value as usize);
    }
    webgl_error::record(state, webgl_constants::INVALID_VALUE);
    None
}
