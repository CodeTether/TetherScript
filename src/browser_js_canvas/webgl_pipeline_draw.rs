//! `drawArrays` validation and software triangle rasterization.

use super::*;

#[path = "webgl_pipeline_draw_build.rs"]
mod build;
#[path = "webgl_pipeline_draw_element_prepare.rs"]
mod element_prepare;
#[path = "webgl_pipeline_draw_elements.rs"]
mod elements;
#[path = "webgl_pipeline_draw_geometry.rs"]
mod geometry;
#[path = "webgl_pipeline_draw_indices.rs"]
mod indices;
#[path = "webgl_pipeline_draw_pixels.rs"]
mod pixels;
#[path = "webgl_pipeline_draw_position.rs"]
mod position;
#[path = "webgl_pipeline_draw_prepare.rs"]
mod prepare;
#[path = "webgl_pipeline_draw_raster.rs"]
mod raster;
#[path = "webgl_pipeline_draw_source.rs"]
mod source;
#[path = "webgl_pipeline_draw_vertex.rs"]
mod vertex_data;
#[path = "webgl_pipeline_draw_validation.rs"]
mod validation;

#[derive(Clone, Copy)]
struct Vertex([f64; 4]);

struct DrawCall {
    vertices: Vec<Vertex>,
    viewport: [i64; 4],
    scissor: Option<[i64; 4]>,
    channels: [bool; 4],
    color: [u8; 4],
}

struct Source(
    shader_state::Program,
    buffer_state::Attribute,
    buffer_state::Buffer,
);

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    let arrays_handle = handle.clone();
    obj.insert(
        "drawArrays".into(),
        native("WebGLRenderingContext.drawArrays", Some(3), move |args| {
            let call =
                webgl_store::mutate(&arrays_handle, version, |state| prepare::call(state, args));
            if let Some(call) = call {
                super::super::store::mutate(&arrays_handle, |surface| raster::draw(surface, &call));
            }
            Ok(JsValue::Undefined)
        }),
    );
    elements::install(obj, handle, version);
}

fn invalid(state: &mut WebGlState) {
    webgl_error::record(state, webgl_constants::INVALID_OPERATION);
}
