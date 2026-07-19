//! Buffer allocation and complete data upload.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "bufferData".into(),
        native("WebGLRenderingContext.bufferData", Some(3), move |args| {
            webgl_store::mutate(&handle, version, |state| upload(state, args));
            Ok(JsValue::Undefined)
        }),
    );
}

fn upload(state: &mut WebGlState, args: &[JsValue]) {
    let Some(target) = validation::target(state, args.first()) else {
        return;
    };
    let usage = webgl_values::u32_value(args.get(2));
    if !constants::usage(usage) {
        webgl_error::record(state, webgl_constants::INVALID_ENUM);
        return;
    }
    let bytes = match source::bytes(args.get(1)) {
        Ok(bytes) => bytes,
        Err(error) => {
            source_error::record(state, error);
            return;
        }
    };
    let Some(bound) = validation::bound(state, target) else {
        return;
    };
    bound.bytes = bytes;
    bound.usage = usage;
}
