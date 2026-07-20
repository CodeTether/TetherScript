//! WebGL unpack alignment and pixel-transform state.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "pixelStorei".into(),
        native("WebGLRenderingContext.pixelStorei", Some(2), move |args| {
            webgl_store::mutate(&handle, version, |state| set(state, args));
            Ok(JsValue::Undefined)
        }),
    );
}

fn set(state: &mut WebGlState, args: &[JsValue]) {
    let name = webgl_values::u32_value(args.first());
    let value = integer(args.get(1));
    match name {
        constants::UNPACK_ALIGNMENT if matches!(value, 1 | 2 | 4 | 8) => {
            state.pipeline.texture_bindings.unpack_alignment = value as u32;
        }
        constants::UNPACK_ALIGNMENT => {
            webgl_error::record(state, webgl_constants::INVALID_VALUE);
        }
        constants::UNPACK_FLIP_Y_WEBGL => state.pipeline.texture_bindings.flip_y = value != 0,
        constants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => {
            state.pipeline.texture_bindings.premultiply_alpha = value != 0;
        }
        _ => webgl_error::record(state, webgl_constants::INVALID_ENUM),
    }
}

fn integer(value: Option<&JsValue>) -> i64 {
    match value {
        Some(JsValue::Bool(value)) => i64::from(*value),
        _ => webgl_values::i64_value(value),
    }
}
