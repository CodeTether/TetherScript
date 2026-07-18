//! Shader compile, deletion, and type parameter queries.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "getShaderParameter".into(),
        native(
            "WebGLRenderingContext.getShaderParameter",
            Some(2),
            move |args| {
                Ok(webgl_store::mutate(&handle, version, |state| {
                    get(state, args)
                }))
            },
        ),
    );
}

fn get(state: &mut WebGlState, args: &[JsValue]) -> JsValue {
    let Some(id) = resource::id(&state.pipeline, args.first(), "shader") else {
        shader::invalid(state);
        return JsValue::Null;
    };
    let Some(resource) = state.pipeline.shaders.get(&id) else {
        shader::invalid(state);
        return JsValue::Null;
    };
    match webgl_values::u32_value(args.get(1)) {
        constants::COMPILE_STATUS => JsValue::Bool(resource.compiled),
        constants::DELETE_STATUS => JsValue::Bool(resource.deleted),
        constants::SHADER_TYPE => JsValue::Number(resource.kind as f64),
        _ => {
            webgl_error::record(state, webgl_constants::INVALID_ENUM);
            JsValue::Null
        }
    }
}
