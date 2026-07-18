//! Program linker information-log query.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "getProgramInfoLog".into(),
        native(
            "WebGLRenderingContext.getProgramInfoLog",
            Some(1),
            move |args| {
                Ok(webgl_store::mutate(&handle, version, |state| {
                    let Some(id) = resource::id(&state.pipeline, args.first(), "program") else {
                        program::invalid(state);
                        return JsValue::Null;
                    };
                    match state.pipeline.programs.get(&id) {
                        Some(program) => JsValue::String(program.log.clone()),
                        None => {
                            program::invalid(state);
                            JsValue::Null
                        }
                    }
                }))
            },
        ),
    );
}
