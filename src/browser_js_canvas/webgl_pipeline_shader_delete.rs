//! Shader deletion and identity tests.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    let delete_handle = handle.clone();
    obj.insert(
        "deleteShader".into(),
        native("WebGLRenderingContext.deleteShader", Some(1), move |args| {
            webgl_store::mutate(&delete_handle, version, |state| {
                if matches!(args.first(), Some(JsValue::Null) | None) {
                    return;
                }
                let Some(id) = resource::id(&state.pipeline, args.first(), "shader") else {
                    shader::invalid(state);
                    return;
                };
                let Some(shader) = state.pipeline.shaders.get_mut(&id) else {
                    shader::invalid(state);
                    return;
                };
                shader.deleted = true;
            });
            Ok(JsValue::Undefined)
        }),
    );
    obj.insert(
        "isShader".into(),
        native("WebGLRenderingContext.isShader", Some(1), move |args| {
            Ok(webgl_store::mutate(&handle, version, |state| {
                let id = resource::id(&state.pipeline, args.first(), "shader");
                JsValue::Bool(
                    id.and_then(|id| state.pipeline.shaders.get(&id))
                        .is_some_and(|shader| !shader.deleted),
                )
            }))
        }),
    );
}
