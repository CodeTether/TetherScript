//! Mutation of one program's attached shader stages.

use super::*;

pub(super) fn attach(state: &mut WebGlState, args: &[JsValue]) {
    let Some(program_id) = resource::id(&state.pipeline, args.first(), "program") else {
        program::invalid(state);
        return;
    };
    let Some(shader_id) = resource::id(&state.pipeline, args.get(1), "shader") else {
        program::invalid(state);
        return;
    };
    let Some(shader) = state
        .pipeline
        .shaders
        .get(&shader_id)
        .filter(|shader| !shader.deleted)
    else {
        program::invalid(state);
        return;
    };
    let kind = shader.kind;
    let Some(linked) = state
        .pipeline
        .programs
        .get_mut(&program_id)
        .filter(|program| !program.deleted)
    else {
        program::invalid(state);
        return;
    };
    if kind == constants::VERTEX_SHADER {
        linked.vertex = Some(shader_id);
    } else {
        linked.fragment = Some(shader_id);
    }
    linked.linked = false;
}
