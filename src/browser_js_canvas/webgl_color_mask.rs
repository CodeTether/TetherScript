//! WebGL color write-mask mutation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "colorMask".into(),
        native("WebGLRenderingContext.colorMask", Some(4), move |args| {
            let mask = [
                args.first().is_some_and(JsValue::truthy),
                args.get(1).is_some_and(JsValue::truthy),
                args.get(2).is_some_and(JsValue::truthy),
                args.get(3).is_some_and(JsValue::truthy),
            ];
            super::webgl_store::mutate(&handle, version, |state| {
                state.color_mask = mask;
                state.push(format!(
                    "colorMask|{}|{}|{}|{}",
                    mask[0], mask[1], mask[2], mask[3]
                ));
            });
            Ok(JsValue::Undefined)
        }),
    );
}
