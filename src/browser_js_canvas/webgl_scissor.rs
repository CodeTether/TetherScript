//! WebGL scissor-box mutation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "scissor".into(),
        native("WebGLRenderingContext.scissor", Some(4), move |args| {
            let box_rect = super::webgl_values::i64_quad(args);
            super::webgl_store::mutate(&handle, version, |state| {
                if box_rect[2] < 0 || box_rect[3] < 0 {
                    super::webgl_error::record(state, super::webgl_constants::INVALID_VALUE);
                    return;
                }
                state.scissor_box = box_rect;
                state.push(format!(
                    "scissor|{}|{}|{}|{}",
                    box_rect[0], box_rect[1], box_rect[2], box_rect[3]
                ));
            });
            Ok(JsValue::Undefined)
        }),
    );
}
