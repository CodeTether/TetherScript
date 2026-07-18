//! WebGL capability enablement queries and mutation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    for (name, enabled) in [("enable", true), ("disable", false)] {
        let handle = handle.clone();
        obj.insert(
            name.into(),
            native(name, Some(1), move |args| {
                let capability = super::webgl_values::u32_value(args.first());
                super::webgl_store::mutate(&handle, version, |state| {
                    set(state, capability, enabled)
                });
                Ok(JsValue::Undefined)
            }),
        );
    }
    obj.insert(
        "isEnabled".into(),
        native("WebGLRenderingContext.isEnabled", Some(1), move |args| {
            let capability = super::webgl_values::u32_value(args.first());
            let enabled = super::webgl_store::mutate(&handle, version, |state| {
                if capability == super::webgl_constants::SCISSOR_TEST {
                    state.scissor_test
                } else {
                    super::webgl_error::record(state, super::webgl_constants::INVALID_ENUM);
                    false
                }
            });
            Ok(JsValue::Bool(enabled))
        }),
    );
}

fn set(state: &mut super::webgl_state::WebGlState, capability: u32, enabled: bool) {
    if capability == super::webgl_constants::SCISSOR_TEST {
        state.scissor_test = enabled;
        state.push(format!(
            "{}|{}",
            if enabled { "enable" } else { "disable" },
            capability
        ));
    } else {
        super::webgl_error::record(state, super::webgl_constants::INVALID_ENUM);
    }
}
