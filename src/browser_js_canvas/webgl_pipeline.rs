//! Dependency-free programmable WebGL pipeline.

pub(super) use super::webgl_state::WebGlState;
use super::*;

macro_rules! pipeline_mod {
    ($path:literal, $name:ident) => {
        #[path = $path]
        mod $name;
    };
}

pipeline_mod!("webgl_pipeline_buffer.rs", buffer);
pipeline_mod!("webgl_pipeline_buffer_state.rs", buffer_state);
pipeline_mod!("webgl_pipeline_draw.rs", draw);
pipeline_mod!("webgl_pipeline_glsl.rs", glsl);
pipeline_mod!("webgl_pipeline_named_constants.rs", named_constants);
pipeline_mod!("webgl_pipeline_params.rs", params);
pipeline_mod!("webgl_pipeline_program.rs", program);
pipeline_mod!("webgl_pipeline_resource.rs", resource);
pipeline_mod!("webgl_pipeline_shader.rs", shader);
pipeline_mod!("webgl_pipeline_shader_state.rs", shader_state);
pipeline_mod!("webgl_pipeline_state.rs", state);
pipeline_mod!("webgl_pipeline_texture.rs", texture);
pipeline_mod!("webgl_pipeline_texture_state.rs", texture_state);
pipeline_mod!("webgl_pipeline_uniform_resource.rs", uniform_resource);
pipeline_mod!("webgl_pipeline_vertex.rs", vertex);

#[path = "webgl_pipeline_constants.rs"]
pub(super) mod constants;

pub(super) use state::State;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    shader::install(obj, handle.clone(), version);
    program::install(obj, handle.clone(), version);
    buffer::install(obj, handle.clone(), version);
    vertex::install(obj, handle.clone(), version);
    texture::install(obj, handle.clone(), version);
    draw::install(obj, handle, version);
}

pub(super) fn install_constants(obj: &mut HashMap<String, JsValue>) {
    named_constants::install(obj);
    texture::install_constants(obj);
}

pub(super) fn parameter(state: &WebGlState, param: u32) -> Option<JsValue> {
    params::get(state, param)
}
