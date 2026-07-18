//! WebGL state serialization onto canvas attributes.

use super::{webgl_state::WebGlState, *};

pub(super) fn sync_attrs(handle: &DomHandle, state: &WebGlState) {
    handle.with_node_mut(|node| {
        if let Node::Element(el) = node {
            el.attrs
                .insert("data-agent-webgl-version".into(), state.version.to_string());
            el.attrs
                .insert("data-agent-webgl-width".into(), state.width.to_string());
            el.attrs
                .insert("data-agent-webgl-height".into(), state.height.to_string());
            super::webgl_attrs_state::sync(el, state);
        }
    });
}
