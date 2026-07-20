//! Validated `ImageData` dimensions and unpacked RGBA pixels.

use super::*;

pub(super) fn decode(
    state: &mut WebGlState,
    source: Option<&JsValue>,
) -> Option<(usize, usize, Vec<[u8; 4]>)> {
    let Some(source) = source else {
        invalid(state);
        return None;
    };
    if !matches!(
        js::get_host_property(source, "__image_data"),
        Ok(JsValue::Bool(true))
    ) {
        invalid(state);
        return None;
    }
    let Ok(width) = js::get_host_property(source, "width") else {
        invalid(state);
        return None;
    };
    let Ok(height) = js::get_host_property(source, "height") else {
        invalid(state);
        return None;
    };
    let (width, height) = image_validation::size(state, Some(&width), Some(&height))?;
    let Ok(data) = js::get_host_property(source, "data") else {
        invalid(state);
        return None;
    };
    match pixels::decode(
        Some(&data), width, height, &state.pipeline.texture_bindings, false,
    ) {
        Ok(pixels) => Some((width, height, pixels)),
        Err(error) => {
            pixels::record(state, error);
            None
        }
    }
}

fn invalid(state: &mut WebGlState) {
    webgl_error::record(state, webgl_constants::INVALID_VALUE);
}
