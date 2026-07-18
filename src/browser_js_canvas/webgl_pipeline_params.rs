//! Programmable-pipeline values exposed through `getParameter`.

use super::*;

pub(super) fn get(state: &WebGlState, param: u32) -> Option<JsValue> {
    match param {
        constants::ARRAY_BUFFER_BINDING => {
            Some(state.pipeline.object(state.pipeline.bound_array_buffer))
        }
        constants::CURRENT_PROGRAM => Some(state.pipeline.object(state.pipeline.current_program)),
        _ => None,
    }
}
