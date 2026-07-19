//! In-place replacement of uploaded buffer byte ranges.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "bufferSubData".into(),
        native(
            "WebGLRenderingContext.bufferSubData",
            Some(3),
            move |args| {
                webgl_store::mutate(&handle, version, |state| replace(state, args));
                Ok(JsValue::Undefined)
            },
        ),
    );
}

fn replace(state: &mut WebGlState, args: &[JsValue]) {
    let Some(target) = validation::target(state, args.first()) else {
        return;
    };
    let bytes = match source::data(args.get(2)) {
        Ok(bytes) => bytes,
        Err(error) => {
            source_error::record(state, error);
            return;
        }
    };
    sub_range::write(state, target, webgl_values::i64_value(args.get(1)), &bytes);
}
