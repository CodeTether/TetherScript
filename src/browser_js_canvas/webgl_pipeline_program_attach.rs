//! Attachment of compiled-stage candidates to WebGL programs.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "attachShader".into(),
        native("WebGLRenderingContext.attachShader", Some(2), move |args| {
            webgl_store::mutate(&handle, version, |state| {
                attachment_state::attach(state, args)
            });
            Ok(JsValue::Undefined)
        }),
    );
}
