//! Program deletion and bound-state cleanup.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "deleteProgram".into(),
        native(
            "WebGLRenderingContext.deleteProgram",
            Some(1),
            move |args| {
                webgl_store::mutate(&handle, version, |state| destroy(state, args.first()));
                Ok(JsValue::Undefined)
            },
        ),
    );
}

fn destroy(state: &mut WebGlState, value: Option<&JsValue>) {
    if matches!(value, None | Some(JsValue::Null)) {
        return;
    }
    let Some(id) = resource::id(&state.pipeline, value, "program") else {
        program::invalid(state);
        return;
    };
    let Some(linked) = state.pipeline.programs.get_mut(&id) else {
        program::invalid(state);
        return;
    };
    linked.deleted = true;
    state
        .pipeline
        .uniform_locations
        .retain(|(program, _), _| *program != id);
    if state.pipeline.current_program == Some(id) {
        state.pipeline.current_program = None;
    }
}
