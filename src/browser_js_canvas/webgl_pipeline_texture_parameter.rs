//! Texture sampling/wrapping parameter mutation and queries.

use super::*;

#[path = "webgl_pipeline_texture_parameter_get.rs"]
mod get;
#[path = "webgl_pipeline_texture_parameter_set.rs"]
mod set;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    let set_handle = handle.clone();
    let float_handle = handle.clone();
    obj.insert(
        "texParameteri".into(),
        native("WebGLRenderingContext.texParameteri", Some(3), move |args| {
            webgl_store::mutate(&set_handle, version, |state| set::value(state, args));
            Ok(JsValue::Undefined)
        }),
    );
    obj.insert(
        "texParameterf".into(),
        native("WebGLRenderingContext.texParameterf", Some(3), move |args| {
            webgl_store::mutate(&float_handle, version, |state| set::value(state, args));
            Ok(JsValue::Undefined)
        }),
    );
    obj.insert(
        "getTexParameter".into(),
        native(
            "WebGLRenderingContext.getTexParameter",
            Some(2),
            move |args| Ok(webgl_store::mutate(&handle, version, |state| get::value(state, args))),
        ),
    );
}
