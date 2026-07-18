//! Linked-program uniform location lookup.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "getUniformLocation".into(),
        native(
            "WebGLRenderingContext.getUniformLocation",
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
    let Some(id) = resource::id(&state.pipeline, args.first(), "program") else {
        program::invalid(state);
        return JsValue::Null;
    };
    let name = args.get(1).map(JsValue::display).unwrap_or_default();
    let exists = state
        .pipeline
        .programs
        .get(&id)
        .is_some_and(|program| program.linked && program.uniforms.contains_key(&name));
    if exists {
        uniform_resource::object(&mut state.pipeline, id, &name)
    } else {
        JsValue::Null
    }
}
