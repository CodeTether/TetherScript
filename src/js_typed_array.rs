//! Typed-array indexed assignment coercion.

use super::*;

pub(super) fn fixed_length(items: &Rc<RefCell<Vec<JsValue>>>) -> bool {
    matches!(
        super::array_extra_property(items, "__typed_array"),
        Some(JsValue::Bool(true))
    )
}

pub(super) fn assigned_value(items: &Rc<RefCell<Vec<JsValue>>>, value: JsValue) -> JsValue {
    match super::array_extra_property(items, "__typed_array_name") {
        Some(JsValue::String(name)) if name == "Uint8ClampedArray" => {
            JsValue::Number(uint8_clamp(&value) as f64)
        }
        _ => value,
    }
}

pub(crate) fn uint8_clamp(value: &JsValue) -> u8 {
    let number = value.number();
    if number.is_nan() || number <= 0.0 {
        return 0;
    }
    if number >= 255.0 {
        return 255;
    }
    let floor = number.floor();
    let round_up = number - floor > 0.5 || (number - floor == 0.5 && floor as u8 % 2 == 1);
    floor as u8 + u8::from(round_up)
}
