//! WebGL drawing-buffer readback modules.

use super::*;

#[path = "webgl_read_area.rs"]
pub(super) mod area;
#[path = "webgl_read_args.rs"]
pub(super) mod args;
#[path = "webgl_read_format.rs"]
pub(super) mod format;
#[path = "webgl_read_pixels.rs"]
pub(super) mod pixels;
#[path = "webgl_readback.rs"]
pub(super) mod readback;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    pixels::install(obj, handle, version);
}
#[path = "webgl_read_target.rs"]
pub(super) mod target;
