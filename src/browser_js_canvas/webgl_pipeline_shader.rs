//! WebGL shader object lifecycle and queries.

use super::*;

#[path = "webgl_pipeline_shader_compile.rs"]
mod compile;
#[path = "webgl_pipeline_shader_create.rs"]
mod create;
#[path = "webgl_pipeline_shader_delete.rs"]
mod delete;
#[path = "webgl_pipeline_shader_query.rs"]
mod query;
#[path = "webgl_pipeline_shader_source.rs"]
mod source;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    create::install(obj, handle.clone(), version);
    source::install(obj, handle.clone(), version);
    compile::install(obj, handle.clone(), version);
    query::install(obj, handle.clone(), version);
    delete::install(obj, handle, version);
}

pub(super) fn invalid(state: &mut WebGlState) {
    webgl_error::record(state, webgl_constants::INVALID_OPERATION);
}
