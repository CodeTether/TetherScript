//! Current linked-program selection.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "useProgram".into(),
        native("WebGLRenderingContext.useProgram", Some(1), move |args| {
            webgl_store::mutate(&handle, version, |state| select(state, args.first()));
            Ok(JsValue::Undefined)
        }),
    );
}

fn select(state: &mut WebGlState, value: Option<&JsValue>) {
    if matches!(value, None | Some(JsValue::Null)) {
        state.pipeline.current_program = None;
        return;
    }
    let Some(id) = resource::id(&state.pipeline, value, "program") else {
        program::invalid(state);
        return;
    };
    if state
        .pipeline
        .programs
        .get(&id)
        .is_some_and(|program| program.linked && !program.deleted)
    {
        state.pipeline.current_program = Some(id);
    } else {
        program::invalid(state);
    }
}
