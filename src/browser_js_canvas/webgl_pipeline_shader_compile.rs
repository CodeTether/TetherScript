//! Deterministic GLSL ES compilation status updates.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "compileShader".into(),
        native(
            "WebGLRenderingContext.compileShader",
            Some(1),
            move |args| {
                webgl_store::mutate(&handle, version, |state| compile(state, args.first()));
                Ok(JsValue::Undefined)
            },
        ),
    );
}

fn compile(state: &mut WebGlState, value: Option<&JsValue>) {
    let Some(id) = resource::id(&state.pipeline, value, "shader") else {
        shader::invalid(state);
        return;
    };
    let Some(resource) = state
        .pipeline
        .shaders
        .get_mut(&id)
        .filter(|shader| !shader.deleted)
    else {
        shader::invalid(state);
        return;
    };
    match glsl::validate(resource.kind, &resource.source) {
        Ok(()) => {
            resource.compiled = true;
            resource.log.clear();
        }
        Err(log) => {
            resource.compiled = false;
            resource.log = log;
        }
    }
}
