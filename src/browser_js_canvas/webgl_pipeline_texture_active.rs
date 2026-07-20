//! Active texture-unit selection.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "activeTexture".into(),
        native("WebGLRenderingContext.activeTexture", Some(1), move |args| {
            webgl_store::mutate(&handle, version, |state| select(state, args.first()));
            Ok(JsValue::Undefined)
        }),
    );
}

fn select(state: &mut WebGlState, value: Option<&JsValue>) {
    let raw = webgl_values::i64_value(value);
    let first = constants::TEXTURE0 as i64;
    let unit = raw.saturating_sub(first);
    if (0..constants::MAX_TEXTURE_UNITS as i64).contains(&unit) {
        state.pipeline.texture_bindings.active = unit as usize;
    } else {
        webgl_error::record(state, webgl_constants::INVALID_ENUM);
    }
}
