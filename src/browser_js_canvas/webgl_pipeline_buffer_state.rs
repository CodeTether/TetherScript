//! Uploaded vertex buffers and attribute pointer state.

#[derive(Clone)]
pub(super) struct Buffer {
    pub bytes: Vec<u8>,
    pub usage: u32,
    pub deleted: bool,
}

#[derive(Clone, Default)]
pub(super) struct Attribute {
    pub enabled: bool,
    pub buffer: Option<u32>,
    pub size: usize,
    pub stride: usize,
    pub offset: usize,
}

impl Buffer {
    pub fn empty() -> Self {
        Self {
            bytes: Vec::new(),
            usage: super::constants::STATIC_DRAW,
            deleted: false,
        }
    }
}
