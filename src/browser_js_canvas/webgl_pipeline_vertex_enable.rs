//! Vertex attribute-array enable and disable operations.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    let enable_handle = handle.clone();
    obj.insert(
        "enableVertexAttribArray".into(),
        native(
            "WebGLRenderingContext.enableVertexAttribArray",
            Some(1),
            move |args| {
                webgl_store::mutate(&enable_handle, version, |state| {
                    set(state, args.first(), true)
                });
                Ok(JsValue::Undefined)
            },
        ),
    );
    obj.insert(
        "disableVertexAttribArray".into(),
        native(
            "WebGLRenderingContext.disableVertexAttribArray",
            Some(1),
            move |args| {
                webgl_store::mutate(&handle, version, |state| set(state, args.first(), false));
                Ok(JsValue::Undefined)
            },
        ),
    );
}

fn set(state: &mut WebGlState, value: Option<&JsValue>, enabled: bool) {
    let Some(location) = vertex::location(state, value) else {
        return;
    };
    state
        .pipeline
        .attributes
        .entry(location)
        .or_default()
        .enabled = enabled;
}
