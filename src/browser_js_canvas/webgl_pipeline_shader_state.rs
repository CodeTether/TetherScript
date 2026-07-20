//! Compiled shader and linked program state.

use super::*;

#[derive(Clone)]
pub(super) struct Shader {
    pub kind: u32,
    pub source: String,
    pub compiled: bool,
    pub deleted: bool,
    pub log: String,
}

#[derive(Clone)]
pub(super) enum ColorSource {
    Constant([f64; 4]),
    Uniform(String),
    Texture {
        sampler: String,
        coordinates: String,
    },
}

#[derive(Clone)]
pub(super) struct Program {
    pub vertex: Option<u32>,
    pub fragment: Option<u32>,
    pub linked: bool,
    pub deleted: bool,
    pub log: String,
    pub attributes: HashMap<String, u32>,
    pub uniforms: HashMap<String, [f64; 4]>,
    pub samplers: HashMap<String, i32>,
    pub color: ColorSource,
}
