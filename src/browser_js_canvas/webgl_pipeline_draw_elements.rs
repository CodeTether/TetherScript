//! `drawElements` entry point and software-raster dispatch.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "drawElements".into(),
        native("WebGLRenderingContext.drawElements", Some(4), move |args| {
            let call =
                webgl_store::mutate(&handle, version, |state| element_prepare::call(state, args));
            if let Some(call) = call {
                super::super::store::mutate(&handle, |surface| raster::draw(surface, &call));
            }
            Ok(JsValue::Undefined)
        }),
    );
}
