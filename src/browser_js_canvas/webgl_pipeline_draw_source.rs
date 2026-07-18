//! Resolution of active program, position attribute, and vertex buffer.

use super::*;

pub(super) fn resolve(state: &mut WebGlState) -> Option<Source> {
    let Some(program_id) = state.pipeline.current_program else {
        draw::invalid(state);
        return None;
    };
    let Some(program) = state
        .pipeline
        .programs
        .get(&program_id)
        .filter(|program| program.linked)
        .cloned()
    else {
        draw::invalid(state);
        return None;
    };
    let Some(location) = position::location(&program) else {
        draw::invalid(state);
        return None;
    };
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
    Some(Source(program, attribute, buffer))
}
