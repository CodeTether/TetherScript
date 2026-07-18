//! Program deletion and identity method installation.

use super::*;

#[path = "webgl_pipeline_program_destroy.rs"]
mod destroy;
#[path = "webgl_pipeline_program_identity.rs"]
mod identity;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    destroy::install(obj, handle.clone(), version);
    identity::install(obj, handle, version);
}
