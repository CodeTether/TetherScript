//! WebGL `getParameter` values.

use super::{webgl_constants as c, webgl_state::WebGlState, *};

pub(super) fn get(state: &mut WebGlState, param: &JsValue) -> JsValue {
    let param = super::webgl_param_value::number(param);
    if let Some(value) = super::webgl_pipeline::parameter(state, param) {
        return value;
    }
    match param {
        c::VENDOR => JsValue::String("tetherscript".into()),
        c::RENDERER => JsValue::String("tetherscript software rasterizer".into()),
        c::VERSION => JsValue::String(format!("WebGL {}.0 (tetherscript)", state.version)),
        c::SHADING_LANGUAGE_VERSION => {
            JsValue::String(super::webgl_param_value::shader_version(state.version))
        }
        c::MAX_TEXTURE_SIZE => JsValue::Number(0.0),
        c::MAX_VIEWPORT_DIMS => {
            super::webgl_param_value::array(&[state.width as f64, state.height as f64])
        }
        c::VIEWPORT => super::webgl_param_value::array(&state.viewport.map(|value| value as f64)),
        c::COLOR_CLEAR_VALUE => super::webgl_param_value::array(&state.clear_color),
        c::DEPTH_CLEAR_VALUE => JsValue::Number(1.0),
        c::STENCIL_CLEAR_VALUE => JsValue::Number(0.0),
        c::SCISSOR_BOX => {
            super::webgl_param_value::array(&state.scissor_box.map(|value| value as f64))
        }
        c::SCISSOR_TEST => JsValue::Bool(state.scissor_test),
        c::COLOR_WRITEMASK => super::webgl_param_value::bool_array(&state.color_mask),
        c::ALIASED_LINE_WIDTH_RANGE => super::webgl_param_value::array(&[1.0, 1.0]),
        c::ALIASED_POINT_SIZE_RANGE => super::webgl_param_value::array(&[1.0, 64.0]),
        c::MAX_VERTEX_ATTRIBS => JsValue::Number(16.0),
        c::MAX_COMBINED_TEXTURE_IMAGE_UNITS
        | c::MAX_TEXTURE_IMAGE_UNITS
        | c::MAX_VERTEX_TEXTURE_IMAGE_UNITS => JsValue::Number(0.0),
        _ => {
            super::webgl_error::record(state, c::INVALID_ENUM);
            JsValue::Null
        }
    }
}
