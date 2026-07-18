//! Validation of a program passed to `linkProgram`.

use super::*;

pub(super) fn id(state: &mut WebGlState, value: Option<&JsValue>) -> Option<u32> {
    let Some(id) = resource::id(&state.pipeline, value, "program") else {
        program::invalid(state);
        return None;
    };
    if state
        .pipeline
        .programs
        .get(&id)
        .is_none_or(|program| program.deleted)
    {
        program::invalid(state);
        return None;
    }
    Some(id)
}
