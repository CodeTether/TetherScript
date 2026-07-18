//! Four-component floating-point uniform updates.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "uniform4f".into(),
        native("WebGLRenderingContext.uniform4f", Some(5), move |args| {
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
    let values = [args.get(1), args.get(2), args.get(3), args.get(4)].map(number);
    let Some(linked) = state.pipeline.programs.get_mut(&id) else {
        program::invalid(state);
        return;
    };
    if let Some(uniform) = linked.uniforms.get_mut(&name) {
        *uniform = values;
    } else {
        program::invalid(state);
    }
}

fn number(value: Option<&JsValue>) -> f64 {
    value
        .map(JsValue::display)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0.0)
}
