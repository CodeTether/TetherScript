//! Shared immutable draw-call assembly from resolved vertex indices.

use super::*;

pub(super) fn call(
    state: &mut WebGlState,
    Source(program, attribute, buffer, coordinates, texture): Source,
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
        let Some(mut vertex) = vertex_data::read(&buffer.bytes, &attribute, index) else {
            draw::invalid(state);
            return None;
        };
        if let Some((attribute, buffer)) = &coordinates {
            let Some(values) = vertex_data::read(&buffer.bytes, attribute, index) else {
                draw::invalid(state);
                return None;
            };
            vertex.1 = Some([values.0[0], values.0[1]]);
        }
        vertices.push(vertex);
    }
    Some(assemble::call(state, program, vertices, texture))
}