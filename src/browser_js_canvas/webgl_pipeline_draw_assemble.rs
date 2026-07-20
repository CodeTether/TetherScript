//! Final draw-call assembly from resolved vertices and fragment state.

use super::*;

pub(super) fn call(
    state: &WebGlState,
    program: shader_state::Program,
    vertices: Vec<Vertex>,
    texture: Option<texture_state::Texture>,
) -> DrawCall {
    let fragment = match &program.color {
        shader_state::ColorSource::Constant(color) => Fragment::Solid(webgl_values::rgba8(*color)),
        shader_state::ColorSource::Uniform(name) => Fragment::Solid(webgl_values::rgba8(
            *program.uniforms.get(name).unwrap_or(&[0.0; 4]),
        )),
        shader_state::ColorSource::Texture { .. } => {
            Fragment::Texture(texture.expect("resolved sampled texture"))
        }
    };
    DrawCall {
        vertices,
        viewport: state.viewport,
        scissor: state.scissor_test.then_some(state.scissor_box),
        channels: state.color_mask,
        fragment,
    }
}