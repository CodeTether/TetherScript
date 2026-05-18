//! Shared request builders.

use std::collections::HashMap;

use super::super::{
    headers_from_value, request_body_from_value, signal_aborted, FetchRequest, JsValue,
};
use super::init::{credentials, mode};

pub(super) fn object(obj: &HashMap<String, JsValue>, fallback: &JsValue) -> FetchRequest {
    FetchRequest {
        url: url(obj, fallback),
        method: field(obj, "method", "GET").to_ascii_uppercase(),
        headers: request_headers(obj),
        body: obj.get("body").and_then(request_body_from_value),
        aborted: obj.get("signal").is_some_and(signal_aborted),
        credentials: credentials(obj.get("credentials")),
        mode: mode(obj.get("mode")),
        initiator_url: String::new(),
    }
}

pub(super) fn string(url: String) -> FetchRequest {
    FetchRequest {
        url,
        method: "GET".into(),
        headers: Vec::new(),
        body: None,
        aborted: false,
        credentials: "same-origin".into(),
        mode: "cors".into(),
        initiator_url: String::new(),
    }
}

fn request_headers(obj: &HashMap<String, JsValue>) -> Vec<(String, String)> {
    obj.get("headers")
        .map(headers_from_value)
        .or_else(|| obj.get("__requestHeaders").map(headers_from_value))
        .unwrap_or_default()
}

fn url(obj: &HashMap<String, JsValue>, fallback: &JsValue) -> String {
    obj.get("url")
        .or_else(|| obj.get("href"))
        .map(JsValue::display)
        .unwrap_or_else(|| fallback.display())
}

fn field(obj: &HashMap<String, JsValue>, name: &str, default: &str) -> String {
    obj.get(name)
        .map(JsValue::display)
        .unwrap_or_else(|| default.into())
}
