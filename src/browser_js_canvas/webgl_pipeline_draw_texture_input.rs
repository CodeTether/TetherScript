//! Resolution of the enabled vertex input that carries texture coordinates.

use super::*;

pub(super) fn resolve(state: &mut WebGlState, location: u32) -> Option<Input> {
    let Some(attribute) = state
        .pipeline
        .attributes
        .get(&location)
        .filter(|attribute| attribute.enabled)
        .cloned()
    else {
        draw::invalid(state);
        return None;
    };
    let Some(buffer) = attribute
        .buffer
        .and_then(|id| state.pipeline.buffers.get(&id))
        .filter(|buffer| !buffer.deleted)
        .cloned()
    else {
        draw::invalid(state);
        return None;
    };
    Some((attribute, buffer))
}