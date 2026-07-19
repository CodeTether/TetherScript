//! Bound buffer size and usage queries.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "getBufferParameter".into(),
        native(
            "WebGLRenderingContext.getBufferParameter",
            Some(2),
            move |args| {
                Ok(webgl_store::mutate(&handle, version, |state| {
                    parameter(state, args)
                }))
            },
        ),
    );
}

fn parameter(state: &mut WebGlState, args: &[JsValue]) -> JsValue {
    let Some(target) = validation::target(state, args.first()) else {
        return JsValue::Null;
    };
    let Some(id) = validation::binding(state, target) else {
        buffer::invalid(state);
        return JsValue::Null;
    };
    let Some(buffer) = state.pipeline.buffers.get(&id) else {
        buffer::invalid(state);
        return JsValue::Null;
    };
    match webgl_values::u32_value(args.get(1)) {
        constants::BUFFER_SIZE => JsValue::Number(buffer.bytes.len() as f64),
        constants::BUFFER_USAGE => JsValue::Number(buffer.usage as f64),
        _ => {
            webgl_error::record(state, webgl_constants::INVALID_ENUM);
            JsValue::Null
        }
    }
}
