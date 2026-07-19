//! Buffer deletion and identity tests.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    let delete_handle = handle.clone();
    obj.insert(
        "deleteBuffer".into(),
        native("WebGLRenderingContext.deleteBuffer", Some(1), move |args| {
            webgl_store::mutate(&delete_handle, version, |state| delete(state, args.first()));
            Ok(JsValue::Undefined)
        }),
    );
    obj.insert(
        "isBuffer".into(),
        native("WebGLRenderingContext.isBuffer", Some(1), move |args| {
            Ok(webgl_store::mutate(&handle, version, |state| {
                let id = resource::id(&state.pipeline, args.first(), "buffer");
                JsValue::Bool(
                    id.and_then(|id| state.pipeline.buffers.get(&id))
                        .is_some_and(|buffer| !buffer.deleted),
                )
            }))
        }),
    );
}

fn delete(state: &mut WebGlState, value: Option<&JsValue>) {
    if matches!(value, None | Some(JsValue::Null)) {
        return;
    }
    let Some(id) = resource::id(&state.pipeline, value, "buffer") else {
        buffer::invalid(state);
        return;
    };
    let Some(resource) = state.pipeline.buffers.get_mut(&id) else {
        buffer::invalid(state);
        return;
    };
    resource.deleted = true;
    if state.pipeline.bound_array_buffer == Some(id) {
        state.pipeline.bound_array_buffer = None;
    }
    if state.pipeline.bound_element_array_buffer == Some(id) {
        state.pipeline.bound_element_array_buffer = None;
    }
}
