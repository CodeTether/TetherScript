//! Canvas `ImageData` method installation.

use super::*;

pub(super) fn install(object: &mut HashMap<String, JsValue>, handle: DomHandle) {
    let get_handle = handle.clone();
    object.insert(
        "getImageData".into(),
        native(
            "CanvasRenderingContext2D.getImageData",
            Some(4),
            move |args| super::snapshot::get(&get_handle, args),
        ),
    );
    object.insert(
        "createImageData".into(),
        native(
            "CanvasRenderingContext2D.createImageData",
            Some(2),
            super::create::blank,
        ),
    );
    let put_handle = handle.clone();
    object.insert(
        "putImageData".into(),
        native(
            "CanvasRenderingContext2D.putImageData",
            Some(3),
            move |args| super::write::put(&put_handle, args),
        ),
    );
    object.insert(
        "__summary".into(),
        native("CanvasRenderingContext2D.__summary", Some(0), move |_| {
            Ok(JsValue::String(super::attrs::summary(&handle)))
        }),
    );
}
