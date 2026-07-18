//! Shared `ARRAY_BUFFER` target and binding validation.

use super::*;

pub(super) fn target(state: &mut WebGlState, value: Option<&JsValue>) -> bool {
    if webgl_values::u32_value(value) == constants::ARRAY_BUFFER {
        return true;
    }
    webgl_error::record(state, webgl_constants::INVALID_ENUM);
    false
}

pub(super) fn bound(state: &mut WebGlState) -> Option<&mut buffer_state::Buffer> {
    let Some(id) = state.pipeline.bound_array_buffer else {
        buffer::invalid(state);
        return None;
    };
    state.pipeline.buffers.get_mut(&id)
}
