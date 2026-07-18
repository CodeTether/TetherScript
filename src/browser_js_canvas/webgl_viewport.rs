//! WebGL viewport state mutation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "viewport".into(),
        native("WebGLRenderingContext.viewport", Some(4), move |args| {
            let viewport = super::webgl_values::i64_quad(args);
            super::webgl_store::mutate(&handle, version, |state| {
                state.viewport = viewport;
                state.push(format!(
                    "viewport|{}|{}|{}|{}",
                    viewport[0], viewport[1], viewport[2], viewport[3]
                ));
            });
            Ok(JsValue::Undefined)
        }),
    );
}
