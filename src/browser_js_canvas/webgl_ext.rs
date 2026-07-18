//! WebGL extension metadata.

use super::*;

pub(super) const SUPPORTED: &[&str] = &[];

pub(super) fn supported_array() -> JsValue {
    JsValue::Array(Rc::new(RefCell::new(
        SUPPORTED
            .iter()
            .map(|name| JsValue::String((*name).into()))
            .collect(),
    )))
}

pub(super) fn extension_object(_name: &str) -> JsValue {
    JsValue::Null
}
