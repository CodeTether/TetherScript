//! Serialization of mutable WebGL state for browser-agent inspection.

use super::{webgl_state::WebGlState, *};

pub(super) fn sync(element: &mut Element, state: &WebGlState) {
    for (name, value) in [
        ("viewport", join(&state.viewport)),
        ("clear-color", join(&state.clear_color)),
        ("scissor-box", join(&state.scissor_box)),
        ("color-mask", join(&state.color_mask)),
    ] {
        element
            .attrs
            .insert(format!("data-agent-webgl-{name}"), value);
    }
    element.attrs.insert(
        "data-agent-webgl-scissor-test".into(),
        state.scissor_test.to_string(),
    );
    element
        .attrs
        .insert("data-agent-webgl-commands".into(), state.commands.join(";"));
    element.attrs.insert(
        "data-agent-webgl-extensions".into(),
        super::webgl_ext::SUPPORTED.join(";"),
    );
}

fn join<T: ToString>(values: &[T; 4]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
