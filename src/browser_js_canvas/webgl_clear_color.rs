//! WebGL color-buffer clear value mutation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "clearColor".into(),
        native("WebGLRenderingContext.clearColor", Some(4), move |args| {
            let color = super::webgl_values::f64_quad(args);
            super::webgl_store::mutate(&handle, version, |state| {
                state.clear_color = color;
                state.push(format!(
                    "clearColor|{}|{}|{}|{}",
                    color[0], color[1], color[2], color[3]
                ));
            });
            Ok(JsValue::Undefined)
        }),
    );
}
