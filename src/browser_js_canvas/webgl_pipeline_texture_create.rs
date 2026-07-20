//! WebGL texture object creation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "createTexture".into(),
        native("WebGLRenderingContext.createTexture", Some(0), move |_| {
            Ok(webgl_store::mutate(&handle, version, |state| {
                let (id, object) = state.pipeline.allocate("texture");
                state
                    .pipeline
                    .textures
                    .insert(id, texture_state::Texture::empty());
                object
            }))
        }),
    );
}
