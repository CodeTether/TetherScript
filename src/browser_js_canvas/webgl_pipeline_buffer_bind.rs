//! `ARRAY_BUFFER` binding validation and mutation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "bindBuffer".into(),
        native("WebGLRenderingContext.bindBuffer", Some(2), move |args| {
            webgl_store::mutate(&handle, version, |state| bind(state, args));
            Ok(JsValue::Undefined)
        }),
    );
}

fn bind(state: &mut WebGlState, args: &[JsValue]) {
    if !validation::target(state, args.first()) {
        return;
    }
    if matches!(args.get(1), None | Some(JsValue::Null)) {
        state.pipeline.bound_array_buffer = None;
        return;
    }
    let Some(id) = resource::id(&state.pipeline, args.get(1), "buffer") else {
        buffer::invalid(state);
        return;
    };
    if state
        .pipeline
        .buffers
        .get(&id)
        .is_some_and(|buffer| !buffer.deleted)
    {
        state.pipeline.bound_array_buffer = Some(id);
    } else {
        buffer::invalid(state);
    }
}
