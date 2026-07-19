//! Shared buffer-target and active-binding validation.

use super::*;

pub(super) fn target(state: &mut WebGlState, value: Option<&JsValue>) -> Option<u32> {
    let target = webgl_values::u32_value(value);
    if matches!(
        target,
        constants::ARRAY_BUFFER | constants::ELEMENT_ARRAY_BUFFER
    ) {
        return Some(target);
    }
    webgl_error::record(state, webgl_constants::INVALID_ENUM);
    None
}

pub(super) fn binding(state: &WebGlState, target: u32) -> Option<u32> {
    match target {
        constants::ARRAY_BUFFER => state.pipeline.bound_array_buffer,
        constants::ELEMENT_ARRAY_BUFFER => state.pipeline.bound_element_array_buffer,
        _ => None,
    }
}

pub(super) fn bound(state: &mut WebGlState, target: u32) -> Option<&mut buffer_state::Buffer> {
    let Some(id) = binding(state, target) else {
        buffer::invalid(state);
        return None;
    };
    state.pipeline.buffers.get_mut(&id)
}
