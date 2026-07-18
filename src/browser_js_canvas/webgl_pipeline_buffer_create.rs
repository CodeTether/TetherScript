//! Buffer creation and `ARRAY_BUFFER` binding.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "createBuffer".into(),
        native("WebGLRenderingContext.createBuffer", Some(0), move |_| {
            Ok(webgl_store::mutate(&handle, version, |state| {
                let (id, object) = state.pipeline.allocate("buffer");
                state
                    .pipeline
                    .buffers
                    .insert(id, buffer_state::Buffer::empty());
                object
            }))
        }),
    );
}
