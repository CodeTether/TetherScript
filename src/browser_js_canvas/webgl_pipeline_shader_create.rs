//! Creation of opaque WebGL shader resources.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "createShader".into(),
        native("WebGLRenderingContext.createShader", Some(1), move |args| {
            let kind = webgl_values::u32_value(args.first());
            Ok(webgl_store::mutate(&handle, version, |state| {
                if !matches!(kind, constants::VERTEX_SHADER | constants::FRAGMENT_SHADER) {
                    webgl_error::record(state, webgl_constants::INVALID_ENUM);
                    return JsValue::Null;
                }
                let (id, object) = state.pipeline.allocate("shader");
                state.pipeline.shaders.insert(
                    id,
                    shader_state::Shader {
                        kind,
                        source: String::new(),
                        compiled: false,
                        deleted: false,
                        log: String::new(),
                    },
                );
                object
            }))
        }),
    );
}
