//! Promise event projection for page diagnostics.

use crate::js::JsValue;

const UNHANDLED_PREFIX: &str = "UnhandledPromiseRejection";

pub(crate) fn unhandled_rejection(value: &JsValue) -> Option<String> {
    let JsValue::Object(object) = value else {
        return None;
    };
    let object = object.borrow();
    let JsValue::String(state) = object.get("__promise_state")? else {
        return None;
    };
    if state != "rejected" {
        return None;
    }
    let reason = object
        .get("__promise_reason")
        .or_else(|| object.get("reason"))
        .unwrap_or(&JsValue::Undefined);
    Some(format!("{UNHANDLED_PREFIX}: {}", reason.display()))
}
