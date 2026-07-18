//! Typed-array source value coercion for mutating methods.

use super::*;

pub(super) fn bytes(target: &JsValue, source: &JsValue) -> Vec<u8> {
    if is_clamped(target) {
        if let JsValue::Array(items) = source {
            return items.borrow().iter().map(js::uint8_clamp).collect();
        }
    }
    bytes::bytes_from_value(source)
}

pub(super) fn byte(target: &JsValue, value: Option<&JsValue>) -> u8 {
    if is_clamped(target) {
        return value.map(js::uint8_clamp).unwrap_or(0);
    }
    number::byte(value)
}

fn is_clamped(value: &JsValue) -> bool {
    matches!(
        js::get_host_property(value, "__typed_array_name"),
        Ok(JsValue::String(name)) if name == "Uint8ClampedArray"
    )
}
