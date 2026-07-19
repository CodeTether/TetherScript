//! Buffer-target binding validation and mutation.

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
    let Some(target) = validation::target(state, args.first()) else {
        return;
    };
    if matches!(args.get(1), None | Some(JsValue::Null)) {
        set(state, target, None);
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
        set(state, target, Some(id));
    } else {
        buffer::invalid(state);
    }
}

fn set(state: &mut WebGlState, target: u32, id: Option<u32>) {
    match target {
        constants::ARRAY_BUFFER => state.pipeline.bound_array_buffer = id,
        constants::ELEMENT_ARRAY_BUFFER => state.pipeline.bound_element_array_buffer = id,
        _ => unreachable!("validated WebGL buffer target"),
    }
}
