//! Reflected attribute and uniform method installation.

use super::*;

#[path = "webgl_pipeline_attrib_location.rs"]
mod attribute;
#[path = "webgl_pipeline_uniform_location.rs"]
mod location;
#[path = "webgl_pipeline_uniform_set.rs"]
mod set;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    attribute::install(obj, handle.clone(), version);
    location::install(obj, handle.clone(), version);
    set::install(obj, handle, version);
}
