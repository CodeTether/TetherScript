//! Draw-call validation and immutable raster work assembly.

use super::*;

pub(super) fn call(state: &mut WebGlState, args: &[JsValue]) -> Option<DrawCall> {
    let mode = webgl_values::u32_value(args.first());
    let first = webgl_values::i64_value(args.get(1));
    let count = webgl_values::i64_value(args.get(2));
    if mode != constants::TRIANGLES {
        webgl_error::record(state, webgl_constants::INVALID_ENUM);
        return None;
    }
    if first < 0 || count < 0 {
        webgl_error::record(state, webgl_constants::INVALID_VALUE);
        return None;
    }
    let Source(program, attribute, buffer) = source::resolve(state)?;
    let mut vertices = Vec::with_capacity(count as usize);
    for index in first..first.saturating_add(count) {
        let Some(vertex) = vertex_data::read(&buffer.bytes, &attribute, index as usize) else {
            draw::invalid(state);
            return None;
        };
        vertices.push(vertex);
    }
    let color = match &program.color {
        shader_state::ColorSource::Constant(color) => *color,
        shader_state::ColorSource::Uniform(name) => {
            *program.uniforms.get(name).unwrap_or(&[0.0; 4])
        }
    };
    state.push(format!("drawArrays|{mode}|{first}|{count}"));
    Some(DrawCall {
        vertices,
        viewport: state.viewport,
        scissor: state.scissor_test.then_some(state.scissor_box),
        channels: state.color_mask,
        color: webgl_values::rgba8(color),
    })
}
