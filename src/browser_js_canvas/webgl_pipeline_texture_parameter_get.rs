//! Querying one sampling parameter from the bound texture.

use super::*;

pub(super) fn value(state: &mut WebGlState, args: &[JsValue]) -> JsValue {
    if !binding::target(state, args.first()) {
        return JsValue::Null;
    }
    let name = webgl_values::u32_value(args.get(1));
    let Some(texture) = binding::get(state) else {
        return JsValue::Null;
    };
    let value = match name {
        constants::TEXTURE_MIN_FILTER => texture.min_filter,
        constants::TEXTURE_MAG_FILTER => texture.mag_filter,
        constants::TEXTURE_WRAP_S => texture.wrap_s,
        constants::TEXTURE_WRAP_T => texture.wrap_t,
        _ => {
            webgl_error::record(state, webgl_constants::INVALID_ENUM);
            return JsValue::Null;
        }
    };
    JsValue::Number(value as f64)
}
