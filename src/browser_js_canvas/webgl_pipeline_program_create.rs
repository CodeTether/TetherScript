//! WebGL program object creation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "createProgram".into(),
        native("WebGLRenderingContext.createProgram", Some(0), move |_| {
            Ok(webgl_store::mutate(&handle, version, |state| {
                let (id, object) = state.pipeline.allocate("program");
                state.pipeline.programs.insert(
                    id,
                    shader_state::Program {
                        vertex: None,
                        fragment: None,
                        linked: false,
                        deleted: false,
                        log: String::new(),
                        attributes: HashMap::new(),
                        uniforms: HashMap::new(),
                        samplers: HashMap::new(),
                        color: shader_state::ColorSource::Constant([0.0; 4]),
                    },
                );
                object
            }))
        }),
    );
}
