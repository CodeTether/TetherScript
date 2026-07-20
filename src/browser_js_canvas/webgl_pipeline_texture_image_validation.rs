//! Shared texture upload dimensions, levels, and pixel-format validation.

use super::*;

pub(super) fn level(state: &mut WebGlState, value: Option<&JsValue>) -> bool {
    if webgl_values::i64_value(value) == 0 {
        return true;
    }
    webgl_error::record(state, webgl_constants::INVALID_VALUE);
    false
}

pub(super) fn size(
    state: &mut WebGlState,
    width: Option<&JsValue>,
    height: Option<&JsValue>,
) -> Option<(usize, usize)> {
    let width = webgl_values::i64_value(width);
    let height = webgl_values::i64_value(height);
    if width >= 0
        && height >= 0
        && width <= constants::MAX_TEXTURE_SIZE as i64
        && height <= constants::MAX_TEXTURE_SIZE as i64
    {
        Some((width as usize, height as usize))
    } else {
        webgl_error::record(state, webgl_constants::INVALID_VALUE);
        None
    }
}

pub(super) fn format(
    state: &mut WebGlState,
    internal: Option<&JsValue>,
    format: Option<&JsValue>,
    kind: Option<&JsValue>,
) -> bool {
    let rgba = webgl_constants::RGBA;
    if webgl_values::u32_value(internal) == rgba
        && webgl_values::u32_value(format) == rgba
        && webgl_values::u32_value(kind) == webgl_constants::UNSIGNED_BYTE
    {
        return true;
    }
    webgl_error::record(state, webgl_constants::INVALID_ENUM);
    false
}
