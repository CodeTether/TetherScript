//! Shared immutable draw-call assembly from resolved vertex indices.

use super::*;

pub(super) fn call(
    state: &mut WebGlState,
    Source(program, attribute, buffer): Source,
    count: usize,
    mut index_at: impl FnMut(usize) -> Option<usize>,
) -> Option<DrawCall> {
    let mut vertices = Vec::new();
    if vertices.try_reserve_exact(count).is_err() {
        webgl_error::record(state, webgl_constants::OUT_OF_MEMORY);
        return None;
    }
    for position in 0..count {
        let Some(index) = index_at(position) else {
            draw::invalid(state);
            return None;
        };
        let Some(vertex) = vertex_data::read(&buffer.bytes, &attribute, index) else {
            draw::invalid(state);
            return None;
        };
        vertices.push(vertex);
    }
    Some(assemble(state, program, vertices))
}

fn assemble(state: &WebGlState, program: shader_state::Program, vertices: Vec<Vertex>) -> DrawCall {
    let color = match &program.color {
        shader_state::ColorSource::Constant(color) => *color,
        shader_state::ColorSource::Uniform(name) => *program.uniforms.get(name).unwrap_or(&[0.0; 4]),
    };
    DrawCall {
        vertices,
        viewport: state.viewport,
        scissor: state.scissor_test.then_some(state.scissor_box),
        channels: state.color_mask,
        color: webgl_values::rgba8(color),
    }
}
