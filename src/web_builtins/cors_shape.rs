//! Response-map assembly for the CORS built-ins.
//!
//! Header names are stored lowercase to match the request map built by
//! `src/http_server_request_map.rs`, so a handler merging these into its own
//! header map cannot end up with both `Vary` and `vary`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Insert one string-valued header.
///
/// # Arguments
///
/// * `headers` — Map being built.
/// * `name` — Lowercase header name.
/// * `value` — Header value.
///
/// # Returns
///
/// Nothing; `headers` gains the entry.
pub(super) fn put(headers: &mut HashMap<String, Value>, name: &str, value: &str) {
    headers.insert(name.to_string(), Value::Str(Rc::new(value.to_string())));
}

/// Wrap a header map as a script value.
///
/// # Arguments
///
/// * `headers` — The assembled headers.
///
/// # Returns
///
/// The map as a `Value::Map`.
pub(super) fn map(headers: HashMap<String, Value>) -> Value {
    Value::Map(Rc::new(RefCell::new(headers)))
}

/// Render a token list as a header value.
///
/// # Arguments
///
/// * `tokens` — Already-normalized tokens.
///
/// # Returns
///
/// The tokens joined with `", "`, which is the list form every
/// `Access-Control-*` header uses.
pub(super) fn join(tokens: &[String]) -> String {
    tokens.join(", ")
}

/// Assemble a response map in the shape `http_serve` consumes.
///
/// # Arguments
///
/// * `status` — HTTP status code.
/// * `headers` — Response headers.
///
/// # Returns
///
/// A map with `status`, `headers`, and an empty `body`.
pub(super) fn response(status: i64, headers: HashMap<String, Value>) -> Value {
    let mut out = HashMap::new();
    out.insert("status".into(), Value::Int(status));
    out.insert("headers".into(), map(headers));
    out.insert("body".into(), Value::Str(Rc::new(String::new())));
    Value::Map(Rc::new(RefCell::new(out)))
}
