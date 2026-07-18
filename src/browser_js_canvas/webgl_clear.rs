//! WebGL drawing-buffer clear implementation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "clear".into(),
        native("WebGLRenderingContext.clear", Some(1), move |args| {
            let mask = super::webgl_values::i64_value(args.first()) as u32;
            if mask & super::webgl_constants::COLOR_BUFFER_BIT != 0 {
                let color = super::webgl_store::with_state(&handle, version, |state| {
                    super::webgl_values::rgba8(state.clear_color)
                });
                super::super::store::mutate(&handle, |surface| surface.clear(color));
            }
            super::webgl_store::mutate(&handle, version, |state| {
                state.push(format!("clear|{}", mask));
            });
            Ok(JsValue::Undefined)
        }),
    );
}
