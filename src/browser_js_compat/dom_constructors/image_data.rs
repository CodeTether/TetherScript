//! Global `ImageData` constructor.

use super::*;

pub(super) fn constructor() -> JsValue {
    super::super::super::canvas_host::image_data_constructor()
}
