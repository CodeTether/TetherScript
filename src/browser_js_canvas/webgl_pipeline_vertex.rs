//! Vertex attribute-array enablement and pointer definitions.

use super::*;

#[path = "webgl_pipeline_vertex_enable.rs"]
mod enable;
#[path = "webgl_pipeline_vertex_format.rs"]
mod format;
#[path = "webgl_pipeline_vertex_pointer.rs"]
mod pointer;

struct Format {
    size: usize,
    stride: usize,
    offset: usize,
}

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    enable::install(obj, handle.clone(), version);
    pointer::install(obj, handle, version);
}

pub(super) fn location(state: &mut WebGlState, value: Option<&JsValue>) -> Option<u32> {
    let raw = webgl_values::i64_value(value);
    if (0..16).contains(&raw) {
        Some(raw as u32)
    } else {
        webgl_error::record(state, webgl_constants::INVALID_VALUE);
        None
    }
}
