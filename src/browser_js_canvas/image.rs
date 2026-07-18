//! Canvas `ImageData` read, construction, and write facade.

use super::*;

#[path = "image_attrs.rs"]
mod attrs;
#[path = "image_create.rs"]
mod create;
#[path = "image_dirty.rs"]
mod dirty;
#[path = "image_methods.rs"]
mod methods;
#[path = "image_object.rs"]
mod object;
#[path = "image_prototype.rs"]
mod prototype;
#[path = "image_size.rs"]
mod size;
#[path = "image_snapshot.rs"]
mod snapshot;
#[path = "image_source.rs"]
mod source;
#[path = "image_source_dimension.rs"]
mod source_dimension;
#[path = "image_source_pixels.rs"]
mod source_pixels;
#[path = "image_write.rs"]
mod write;

pub(super) fn constructor() -> JsValue {
    prototype::constructor()
}

pub(super) fn install(object: &mut HashMap<String, JsValue>, handle: DomHandle) {
    methods::install(object, handle);
}

pub(super) fn sync_attrs(handle: &DomHandle, surface: &super::surface::Surface) {
    attrs::sync_attrs(handle, surface);
}
