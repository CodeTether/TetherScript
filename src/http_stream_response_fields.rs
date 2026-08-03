//! Per-field extraction for the streaming-response map.
//!
//! Split from [`super::shape`] so validation of individual keys stays separate
//! from assembling the specification, and so each file stays inside the 50-line
//! limit. Every error names the key that was wrong, per the repository's
//! error-message rule.

use std::collections::HashMap;

use crate::value::Value;

/// Read the `status` key, defaulting to `200`.
///
/// # Arguments
///
/// * `map` — Borrowed streaming-response map.
///
/// # Returns
///
/// The status code.
///
/// # Errors
///
/// Returns `Err` when `status` is present but not an int, or is outside
/// `100..=599`: a status line the client would reject is worse than a default.
pub(crate) fn status(map: &HashMap<String, Value>) -> Result<u16, String> {
    match map.get("status") {
        None | Some(Value::Nil) => Ok(200),
        Some(Value::Int(code)) if (100..=599).contains(code) => Ok(*code as u16),
        Some(Value::Int(code)) => Err(format!(
            "http_serve: stream response.status {code} is outside 100..=599"
        )),
        Some(other) => Err(format!(
            "http_serve: stream response.status must be int, got {}",
            other.type_name()
        )),
    }
}

/// Read the `stream` generator.
///
/// # Arguments
///
/// * `map` — Borrowed streaming-response map.
///
/// # Returns
///
/// A clone of the callable. Cloning a `Value` is an `Rc` bump, so the generator
/// outlives the map borrow without copying the closure.
///
/// # Errors
///
/// Returns `Err` naming `stream` when the key is absent or not a function.
pub(crate) fn generator(map: &HashMap<String, Value>) -> Result<Value, String> {
    match map.get("stream") {
        Some(value @ (Value::Fn(_) | Value::VmFn(_) | Value::Native(_))) => Ok(value.clone()),
        Some(other) => Err(format!(
            "http_serve: stream response.stream must be a zero-argument fn, got {}",
            other.type_name()
        )),
        None => Err("http_serve: stream response is missing the stream key".to_string()),
    }
}

/// Read the `headers` key and apply the SSE defaults.
///
/// `content-type: text/event-stream` and `cache-control: no-cache` are seeded
/// only when the handler did not set them, so an explicit choice always wins.
///
/// # Arguments
///
/// * `map` — Borrowed streaming-response map.
///
/// # Returns
///
/// Lower-cased header names mapped to their rendered values. Infallible: a
/// `headers` value that is not a map contributes nothing rather than failing,
/// matching the ordinary response path's tolerance.
pub(crate) fn headers(map: &HashMap<String, Value>) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    if let Some(Value::Map(given)) = map.get("headers") {
        for (name, value) in given.borrow().iter() {
            headers.insert(name.to_ascii_lowercase(), value.to_string());
        }
    }
    headers
        .entry("content-type".to_string())
        .or_insert_with(|| "text/event-stream; charset=utf-8".to_string());
    headers
        .entry("cache-control".to_string())
        .or_insert_with(|| "no-cache".to_string());
    headers
}
