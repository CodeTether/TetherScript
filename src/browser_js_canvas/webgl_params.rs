//! WebGL `getParameter` values.

use super::{webgl_constants as c, webgl_state::WebGlState, *};

pub(super) fn get(state: &WebGlState, param: &JsValue) -> JsValue {
    match number(param) {
        c::VENDOR => JsValue::String("tetherscript".into()),
        c::RENDERER => JsValue::String("tetherscript software rasterizer".into()),
        c::VERSION => JsValue::String(format!("WebGL {}.0 (tetherscript)", state.version)),
        c::SHADING_LANGUAGE_VERSION => JsValue::Null,
        c::MAX_TEXTURE_SIZE => JsValue::Number(0.0),
        c::MAX_VIEWPORT_DIMS => array(&[state.width as f64, state.height as f64]),
        c::VIEWPORT => array(&state.viewport.map(|value| value as f64)),
        c::COLOR_CLEAR_VALUE => array(&state.clear_color),
        c::DEPTH_CLEAR_VALUE => JsValue::Number(1.0),
        c::STENCIL_CLEAR_VALUE => JsValue::Number(0.0),
        c::ALIASED_LINE_WIDTH_RANGE | c::ALIASED_POINT_SIZE_RANGE => array(&[0.0, 0.0]),
        c::MAX_VERTEX_ATTRIBS
        | c::MAX_COMBINED_TEXTURE_IMAGE_UNITS
        | c::MAX_TEXTURE_IMAGE_UNITS
        | c::MAX_VERTEX_TEXTURE_IMAGE_UNITS => JsValue::Number(0.0),
        _ => JsValue::Null,
    }
}

fn number(value: &JsValue) -> u32 {
    match value {
        JsValue::Number(n) if n.is_finite() && *n >= 0.0 => *n as u32,
        other => other.display().parse().unwrap_or(0),
    }
}

fn array(values: &[f64]) -> JsValue {
    JsValue::Array(Rc::new(RefCell::new(
        values.iter().map(|value| JsValue::Number(*value)).collect(),
    )))
}
