//! RequestInit field application.

use super::super::{headers_from_value, signal_aborted, FetchRequest, JsValue};

pub(super) fn apply(request: &mut FetchRequest, init: &JsValue) {
    let JsValue::Object(obj) = init else {
        return;
    };
    let obj = obj.borrow();
    if let Some(method) = obj.get("method") {
        request.method = method.display().to_ascii_uppercase();
    }
    if let Some(headers) = obj.get("headers") {
        request.headers = headers_from_value(headers);
    }
    if let Some(body) = obj.get("body") {
        request.body = Some(body.display());
    }
    if let Some(signal) = obj.get("signal") {
        request.aborted = signal_aborted(signal);
    }
    request.credentials = credentials(obj.get("credentials"));
    request.mode = mode(obj.get("mode"));
}

pub(super) fn credentials(value: Option<&JsValue>) -> String {
    match value.map(JsValue::display).unwrap_or_default().as_str() {
        "omit" | "include" => value.unwrap().display(),
        _ => "same-origin".into(),
    }
}

pub(super) fn mode(value: Option<&JsValue>) -> String {
    match value.map(JsValue::display).unwrap_or_default().as_str() {
        "same-origin" | "no-cors" => value.unwrap().display(),
        _ => "cors".into(),
    }
}
