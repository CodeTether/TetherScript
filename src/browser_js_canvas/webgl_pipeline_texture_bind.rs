//! `TEXTURE_2D` binding on the active texture unit.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "bindTexture".into(),
        native("WebGLRenderingContext.bindTexture", Some(2), move |args| {
            webgl_store::mutate(&handle, version, |state| bind(state, args));
            Ok(JsValue::Undefined)
        }),
    );
}

fn bind(state: &mut WebGlState, args: &[JsValue]) {
    if !binding::target(state, args.first()) {
        return;
    }
    let unit = state.pipeline.texture_bindings.active;
    if matches!(args.get(1), None | Some(JsValue::Null)) {
        state.pipeline.texture_bindings.units[unit] = None;
        return;
    }
    let Some(id) = resource::id(&state.pipeline, args.get(1), "texture") else {
        texture::invalid(state);
        return;
    };
    if state
        .pipeline
        .textures
        .get(&id)
        .is_some_and(|texture| !texture.deleted)
    {
        state.pipeline.texture_bindings.units[unit] = Some(id);
    } else {
        texture::invalid(state);
    }
}
