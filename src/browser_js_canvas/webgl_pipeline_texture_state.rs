//! Texture pixels, sampling parameters, and texture-unit bindings.

use super::constants;

#[derive(Clone)]
pub(super) struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[u8; 4]>,
    pub min_filter: u32,
    pub mag_filter: u32,
    pub wrap_s: u32,
    pub wrap_t: u32,
    pub deleted: bool,
}

impl Texture {
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            pixels: Vec::new(),
            min_filter: constants::NEAREST,
            mag_filter: constants::LINEAR,
            wrap_s: constants::REPEAT,
            wrap_t: constants::REPEAT,
            deleted: false,
        }
    }
}

#[derive(Clone)]
pub(super) struct Bindings {
    pub active: usize,
    pub units: [Option<u32>; constants::MAX_TEXTURE_UNITS],
    pub unpack_alignment: u32,
    pub flip_y: bool,
    pub premultiply_alpha: bool,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            active: 0,
            units: [None; constants::MAX_TEXTURE_UNITS],
            unpack_alignment: 4,
            flip_y: false,
            premultiply_alpha: false,
        }
    }
}
