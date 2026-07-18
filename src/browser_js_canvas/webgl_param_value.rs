//! JavaScript value construction for WebGL state parameters.

use super::*;

pub(super) fn shader_version(version: u8) -> String {
    if version >= 2 {
        "WebGL GLSL ES 3.00 (tetherscript)"
    } else {
        "WebGL GLSL ES 1.0 (tetherscript)"
    }
    .into()
}

pub(super) fn bool_array(values: &[bool]) -> JsValue {
    JsValue::Array(Rc::new(RefCell::new(
        values.iter().map(|value| JsValue::Bool(*value)).collect(),
    )))
}

pub(super) fn number(value: &JsValue) -> u32 {
    match value {
        JsValue::Number(n) if n.is_finite() && *n >= 0.0 => *n as u32,
        other => other.display().parse().unwrap_or(0),
    }
}

pub(super) fn array(values: &[f64]) -> JsValue {
    JsValue::Array(Rc::new(RefCell::new(
        values.iter().map(|value| JsValue::Number(*value)).collect(),
    )))
}
