//! WebGL program linking, use, reflection, and deletion.

use super::*;

#[path = "webgl_pipeline_program_attach.rs"]
mod attach;
#[path = "webgl_pipeline_program_attachment_state.rs"]
mod attachment_state;
#[path = "webgl_pipeline_program_create.rs"]
mod create;
#[path = "webgl_pipeline_program_delete.rs"]
mod delete;
#[path = "webgl_pipeline_program_link.rs"]
mod link;
#[path = "webgl_pipeline_program_link_target.rs"]
mod link_target;
#[path = "webgl_pipeline_program_query.rs"]
mod query;
#[path = "webgl_pipeline_program_reflection.rs"]
mod reflection;
#[path = "webgl_pipeline_program_uniform.rs"]
mod uniform;
#[path = "webgl_pipeline_program_use.rs"]
mod use_program;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    create::install(obj, handle.clone(), version);
    attach::install(obj, handle.clone(), version);
    link::install(obj, handle.clone(), version);
    use_program::install(obj, handle.clone(), version);
    query::install(obj, handle.clone(), version);
    uniform::install(obj, handle.clone(), version);
    delete::install(obj, handle, version);
}

pub(super) fn invalid(state: &mut WebGlState) {
    webgl_error::record(state, webgl_constants::INVALID_OPERATION);
}
