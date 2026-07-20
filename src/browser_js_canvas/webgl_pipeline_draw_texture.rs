//! Sampled-texture source resolution and fragment color evaluation.

use super::*;

#[path = "webgl_pipeline_draw_texture_input.rs"]
mod input;
#[path = "webgl_pipeline_draw_texture_interpolate.rs"]
mod interpolate;
#[path = "webgl_pipeline_draw_texture_filter.rs"]
mod filter;
#[path = "webgl_pipeline_draw_texture_source.rs"]
mod source;

type Input = (buffer_state::Attribute, buffer_state::Buffer);
type Resolved = (Option<Input>, Option<texture_state::Texture>);

pub(super) fn source(
    state: &mut WebGlState,
    program: &shader_state::Program,
) -> Option<Resolved> {
    source::resolve(state, program)
}

pub(super) fn color(
    call: &DrawCall,
    vertices: &[Vertex],
    points: [[f64; 2]; 3],
    weights: [f64; 3],
) -> [u8; 4] {
    match &call.fragment {
        Fragment::Solid(color) => *color,
        Fragment::Texture(texture) => interpolate::coordinates(vertices, weights)
            .map(|uv| {
                let sampling = filter::value(texture, vertices, points);
                super::super::texture::sample(texture, uv, sampling)
            })
            .unwrap_or([0, 0, 0, 255]),
    }
}