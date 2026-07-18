//! Program object identity query.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "isProgram".into(),
        native("WebGLRenderingContext.isProgram", Some(1), move |args| {
            Ok(webgl_store::mutate(&handle, version, |state| {
                let id = resource::id(&state.pipeline, args.first(), "program");
                JsValue::Bool(
                    id.and_then(|id| state.pipeline.programs.get(&id))
                        .is_some_and(|program| !program.deleted),
                )
            }))
        }),
    );
}
