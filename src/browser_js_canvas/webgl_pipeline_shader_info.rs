//! Shader compiler information-log query.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "getShaderInfoLog".into(),
        native(
            "WebGLRenderingContext.getShaderInfoLog",
            Some(1),
            move |args| {
                Ok(webgl_store::mutate(&handle, version, |state| {
                    let Some(id) = resource::id(&state.pipeline, args.first(), "shader") else {
                        shader::invalid(state);
                        return JsValue::Null;
                    };
                    match state.pipeline.shaders.get(&id) {
                        Some(shader) => JsValue::String(shader.log.clone()),
                        None => {
                            shader::invalid(state);
                            JsValue::Null
                        }
                    }
                }))
            },
        ),
    );
}
