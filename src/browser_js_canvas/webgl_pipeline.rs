//! Dependency-free programmable WebGL pipeline.

pub(super) use super::webgl_state::WebGlState;
use super::*;

#[path = "webgl_pipeline_buffer.rs"]
mod buffer;
#[path = "webgl_pipeline_buffer_state.rs"]
mod buffer_state;
#[path = "webgl_pipeline_constants.rs"]
pub(super) mod constants;
#[path = "webgl_pipeline_draw.rs"]
mod draw;
#[path = "webgl_pipeline_glsl.rs"]
mod glsl;
#[path = "webgl_pipeline_named_constants.rs"]
mod named_constants;
#[path = "webgl_pipeline_params.rs"]
mod params;
#[path = "webgl_pipeline_program.rs"]
mod program;
#[path = "webgl_pipeline_resource.rs"]
mod resource;
#[path = "webgl_pipeline_shader.rs"]
mod shader;
#[path = "webgl_pipeline_shader_state.rs"]
mod shader_state;
#[path = "webgl_pipeline_state.rs"]
mod state;
#[path = "webgl_pipeline_uniform_resource.rs"]
mod uniform_resource;
#[path = "webgl_pipeline_vertex.rs"]
mod vertex;

pub(super) use state::State;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    shader::install(obj, handle.clone(), version);
    program::install(obj, handle.clone(), version);
    buffer::install(obj, handle.clone(), version);
    vertex::install(obj, handle.clone(), version);
    draw::install(obj, handle, version);
}

pub(super) fn install_constants(obj: &mut HashMap<String, JsValue>) {
    named_constants::install(obj);
}

pub(super) fn parameter(state: &WebGlState, param: u32) -> Option<JsValue> {
    params::get(state, param)
}
