//! Linked-program query method installation.

use super::*;

#[path = "webgl_pipeline_program_info.rs"]
mod info;
#[path = "webgl_pipeline_program_parameter.rs"]
mod parameter;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    parameter::install(obj, handle.clone(), version);
    info::install(obj, handle, version);
}
