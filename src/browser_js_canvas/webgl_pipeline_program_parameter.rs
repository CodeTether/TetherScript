//! Linked-program status and active-resource counts.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "getProgramParameter".into(),
        native(
            "WebGLRenderingContext.getProgramParameter",
            Some(2),
            move |args| {
                Ok(webgl_store::mutate(&handle, version, |state| {
                    get(state, args)
                }))
            },
        ),
    );
}

fn get(state: &mut WebGlState, args: &[JsValue]) -> JsValue {
    let Some(id) = resource::id(&state.pipeline, args.first(), "program") else {
        program::invalid(state);
        return JsValue::Null;
    };
    let Some(linked) = state.pipeline.programs.get(&id) else {
        program::invalid(state);
        return JsValue::Null;
    };
    match webgl_values::u32_value(args.get(1)) {
        constants::LINK_STATUS => JsValue::Bool(linked.linked),
        constants::DELETE_STATUS => JsValue::Bool(linked.deleted),
        constants::ATTACHED_SHADERS => JsValue::Number(
            (linked.vertex.is_some() as u8 + linked.fragment.is_some() as u8) as f64,
        ),
        constants::ACTIVE_ATTRIBUTES => JsValue::Number(linked.attributes.len() as f64),
        constants::ACTIVE_UNIFORMS => {
            JsValue::Number((linked.uniforms.len() + linked.samplers.len()) as f64)
        }
        _ => {
            webgl_error::record(state, webgl_constants::INVALID_ENUM);
            JsValue::Null
        }
    }
}
