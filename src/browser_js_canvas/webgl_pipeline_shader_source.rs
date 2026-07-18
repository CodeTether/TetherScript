//! Shader source upload.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "shaderSource".into(),
        native("WebGLRenderingContext.shaderSource", Some(2), move |args| {
            let source = args.get(1).map(JsValue::display).unwrap_or_default();
            webgl_store::mutate(&handle, version, |state| {
                let Some(id) = resource::id(&state.pipeline, args.first(), "shader") else {
                    shader::invalid(state);
                    return;
                };
                let Some(shader) = state
                    .pipeline
                    .shaders
                    .get_mut(&id)
                    .filter(|shader| !shader.deleted)
                else {
                    shader::invalid(state);
                    return;
                };
                shader.source = source;
                shader.compiled = false;
                shader.log.clear();
            });
            Ok(JsValue::Undefined)
        }),
    );
}
