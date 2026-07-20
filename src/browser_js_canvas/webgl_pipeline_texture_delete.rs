//! Texture deletion and identity queries.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    let delete_handle = handle.clone();
    obj.insert(
        "deleteTexture".into(),
        native("WebGLRenderingContext.deleteTexture", Some(1), move |args| {
            webgl_store::mutate(&delete_handle, version, |state| delete(state, args.first()));
            Ok(JsValue::Undefined)
        }),
    );
    obj.insert(
        "isTexture".into(),
        native("WebGLRenderingContext.isTexture", Some(1), move |args| {
            Ok(webgl_store::mutate(&handle, version, |state| {
                let id = resource::id(&state.pipeline, args.first(), "texture");
                JsValue::Bool(id.and_then(|id| state.pipeline.textures.get(&id)).is_some_and(
                    |texture| !texture.deleted,
                ))
            }))
        }),
    );
}

fn delete(state: &mut WebGlState, value: Option<&JsValue>) {
    if matches!(value, None | Some(JsValue::Null)) {
        return;
    }
    let Some(id) = resource::id(&state.pipeline, value, "texture") else {
        texture::invalid(state);
        return;
    };
    let Some(resource) = state.pipeline.textures.get_mut(&id) else {
        texture::invalid(state);
        return;
    };
    resource.deleted = true;
    for binding in &mut state.pipeline.texture_bindings.units {
        if *binding == Some(id) {
            *binding = None;
        }
    }
}
