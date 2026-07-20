//! Integer sampler-uniform updates.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "uniform1i".into(),
        native("WebGLRenderingContext.uniform1i", Some(2), move |args| {
            webgl_store::mutate(&handle, version, |state| set(state, args));
            Ok(JsValue::Undefined)
        }),
    );
}

fn set(state: &mut WebGlState, args: &[JsValue]) {
    if matches!(args.first(), Some(JsValue::Null) | None) {
        return;
    }
    let Some((id, name)) = uniform_resource::parse(&state.pipeline, args.first()) else {
        program::invalid(state);
        return;
    };
    if state.pipeline.current_program != Some(id) {
        program::invalid(state);
        return;
    }
    let value = webgl_values::i64_value(args.get(1)) as i32;
    let Some(linked) = state.pipeline.programs.get_mut(&id) else {
        program::invalid(state);
        return;
    };
    if let Some(sampler) = linked.samplers.get_mut(&name) {
        *sampler = value;
    } else {
        program::invalid(state);
    }
}
