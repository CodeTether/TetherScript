//! Validation and mutation for one bound texture parameter.

use super::*;

pub(super) fn value(state: &mut WebGlState, args: &[JsValue]) {
    if !binding::target(state, args.first()) {
        return;
    }
    let name = webgl_values::u32_value(args.get(1));
    let value = webgl_values::u32_value(args.get(2));
    if !valid(name, value) {
        webgl_error::record(state, webgl_constants::INVALID_ENUM);
        return;
    }
    let Some(texture) = binding::get_mut(state) else {
        return;
    };
    match name {
        constants::TEXTURE_MIN_FILTER => texture.min_filter = value,
        constants::TEXTURE_MAG_FILTER => texture.mag_filter = value,
        constants::TEXTURE_WRAP_S => texture.wrap_s = value,
        constants::TEXTURE_WRAP_T => texture.wrap_t = value,
        _ => unreachable!("validated texture parameter"),
    }
}

fn valid(name: u32, value: u32) -> bool {
    match name {
        constants::TEXTURE_MIN_FILTER | constants::TEXTURE_MAG_FILTER => {
            matches!(value, constants::NEAREST | constants::LINEAR)
        }
        constants::TEXTURE_WRAP_S | constants::TEXTURE_WRAP_T => {
            matches!(value, constants::REPEAT | constants::CLAMP_TO_EDGE)
        }
        _ => false,
    }
}
