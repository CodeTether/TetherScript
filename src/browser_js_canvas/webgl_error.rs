//! Sticky WebGL error state and `getError`.

use super::*;

pub(super) fn record(state: &mut super::webgl_state::WebGlState, error: u32) {
    if error != super::webgl_constants::NO_ERROR && !state.errors.contains(&error) {
        state.errors.push(error);
    }
}

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "getError".into(),
        native("WebGLRenderingContext.getError", Some(0), move |_| {
            let error = super::webgl_store::mutate(&handle, version, |state| {
                if state.errors.is_empty() {
                    super::webgl_constants::NO_ERROR
                } else {
                    state.errors.remove(0)
                }
            });
            Ok(JsValue::Number(error as f64))
        }),
    );
}
