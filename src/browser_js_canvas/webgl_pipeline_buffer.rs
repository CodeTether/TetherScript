//! WebGL vertex-buffer lifecycle, upload, and queries.

use super::*;

#[path = "webgl_pipeline_buffer_bind.rs"]
mod bind;
#[path = "webgl_pipeline_buffer_create.rs"]
mod create;
#[path = "webgl_pipeline_buffer_data.rs"]
mod data;
#[path = "webgl_pipeline_buffer_delete.rs"]
mod delete;
#[path = "webgl_pipeline_buffer_query.rs"]
mod query;
#[path = "webgl_pipeline_buffer_source.rs"]
mod source;
#[path = "webgl_pipeline_buffer_source_error.rs"]
mod source_error;
#[path = "webgl_pipeline_buffer_sub_data.rs"]
mod sub_data;
#[path = "webgl_pipeline_buffer_sub_range.rs"]
mod sub_range;
#[path = "webgl_pipeline_buffer_typed_bytes.rs"]
mod typed_bytes;
#[path = "webgl_pipeline_buffer_validation.rs"]
mod validation;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    create::install(obj, handle.clone(), version);
    bind::install(obj, handle.clone(), version);
    data::install(obj, handle.clone(), version);
    sub_data::install(obj, handle.clone(), version);
    query::install(obj, handle.clone(), version);
    delete::install(obj, handle, version);
}

pub(super) fn invalid(state: &mut WebGlState) {
    webgl_error::record(state, webgl_constants::INVALID_OPERATION);
}
