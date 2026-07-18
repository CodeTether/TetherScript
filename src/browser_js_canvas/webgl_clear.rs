//! WebGL drawing-buffer clear implementation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "clear".into(),
        native("WebGLRenderingContext.clear", Some(1), move |args| {
            let mask = super::webgl_values::i64_value(args.first()) as u32;
            let clear = super::webgl_store::mutate(&handle, version, |state| {
                if mask & !allowed_mask() != 0 {
                    super::webgl_error::record(state, super::webgl_constants::INVALID_VALUE);
                    return None;
                }
                state.push(format!("clear|{}", mask));
                (mask & super::webgl_constants::COLOR_BUFFER_BIT != 0).then_some((
                    super::webgl_values::rgba8(state.clear_color),
                    state.color_mask,
                    state.scissor_test.then_some(state.scissor_box),
                ))
            });
            if let Some((color, channels, scissor)) = clear {
                super::super::store::mutate(&handle, |surface| {
                    super::webgl_clear_region::apply(surface, color, channels, scissor)
                });
            }
            Ok(JsValue::Undefined)
        }),
    );
}

fn allowed_mask() -> u32 {
    super::webgl_constants::COLOR_BUFFER_BIT
        | super::webgl_constants::DEPTH_BUFFER_BIT
        | super::webgl_constants::STENCIL_BUFFER_BIT
}
