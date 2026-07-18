//! Linked-program vertex attribute location lookup.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "getAttribLocation".into(),
        native(
            "WebGLRenderingContext.getAttribLocation",
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
        return JsValue::Number(-1.0);
    };
    let name = args.get(1).map(JsValue::display).unwrap_or_default();
    state
        .pipeline
        .programs
        .get(&id)
        .and_then(|program| program.linked.then_some(program))
        .and_then(|program| program.attributes.get(&name))
        .map_or(JsValue::Number(-1.0), |location| {
            JsValue::Number(*location as f64)
        })
}
