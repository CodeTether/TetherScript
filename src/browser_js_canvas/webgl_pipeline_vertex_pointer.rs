//! `vertexAttribPointer` validation and state capture.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "vertexAttribPointer".into(),
        native(
            "WebGLRenderingContext.vertexAttribPointer",
            Some(6),
            move |args| {
                webgl_store::mutate(&handle, version, |state| pointer(state, args));
                Ok(JsValue::Undefined)
            },
        ),
    );
}

fn pointer(state: &mut WebGlState, args: &[JsValue]) {
    let Some(location) = vertex::location(state, args.first()) else {
        return;
    };
    let Some(format) = format::parse(state, args) else {
        return;
    };
    let Some(buffer) = state.pipeline.bound_array_buffer else {
        buffer::invalid(state);
        return;
    };
    let attribute = state.pipeline.attributes.entry(location).or_default();
    attribute.buffer = Some(buffer);
    attribute.size = format.size;
    attribute.stride = format.stride;
    attribute.offset = format.offset;
}
