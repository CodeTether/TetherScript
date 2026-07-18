//! WebGL method installation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    super::webgl_viewport::install(obj, handle.clone(), version);
    super::webgl_clear_color::install(obj, handle.clone(), version);
    super::webgl_clear::install(obj, handle.clone(), version);
    super::webgl_read::install(obj, handle, version);
}
