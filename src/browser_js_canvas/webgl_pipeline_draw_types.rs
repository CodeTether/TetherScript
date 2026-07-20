//! Shared data passed between WebGL draw preparation and rasterization.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct Vertex(pub(super) [f64; 4], pub(super) Option<[f64; 2]>);

pub(super) enum Fragment {
    Solid([u8; 4]),
    Texture(texture_state::Texture),
}

pub(super) struct DrawCall {
    pub(super) vertices: Vec<Vertex>,
    pub(super) viewport: [i64; 4],
    pub(super) scissor: Option<[i64; 4]>,
    pub(super) channels: [bool; 4],
    pub(super) fragment: Fragment,
}

pub(super) struct Source(
    pub(super) shader_state::Program,
    pub(super) buffer_state::Attribute,
    pub(super) buffer_state::Buffer,
    pub(super) Option<(buffer_state::Attribute, buffer_state::Buffer)>,
    pub(super) Option<texture_state::Texture>,
);
