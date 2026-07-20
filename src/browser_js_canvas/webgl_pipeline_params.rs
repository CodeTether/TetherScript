//! Programmable-pipeline values exposed through `getParameter`.

use super::*;

pub(super) fn get(state: &WebGlState, param: u32) -> Option<JsValue> {
    match param {
        constants::ARRAY_BUFFER_BINDING => {
            Some(state.pipeline.object(state.pipeline.bound_array_buffer))
        }
        constants::ELEMENT_ARRAY_BUFFER_BINDING => Some(
            state
                .pipeline
                .object(state.pipeline.bound_element_array_buffer),
        ),
        constants::ACTIVE_TEXTURE => Some(JsValue::Number(
            (constants::TEXTURE0 as usize + state.pipeline.texture_bindings.active) as f64,
        )),
        constants::TEXTURE_BINDING_2D => {
            let bindings = &state.pipeline.texture_bindings;
            Some(state.pipeline.object(bindings.units[bindings.active]))
        }
        constants::UNPACK_ALIGNMENT => Some(JsValue::Number(
            state.pipeline.texture_bindings.unpack_alignment as f64,
        )),
        constants::UNPACK_FLIP_Y_WEBGL => {
            Some(JsValue::Bool(state.pipeline.texture_bindings.flip_y))
        }
        constants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => Some(JsValue::Bool(
            state.pipeline.texture_bindings.premultiply_alpha,
        )),
        constants::CURRENT_PROGRAM => Some(state.pipeline.object(state.pipeline.current_program)),
        _ => None,
    }
}
