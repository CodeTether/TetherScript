//! Field extraction for a streaming response map.

use std::collections::HashMap;

use crate::value::Value;

/// Read an optional boolean field, defaulting to false.
///
/// # Errors
///
/// Returns an error naming the field when it is present but not a bool, rather than
/// silently treating a typo'd value as false.
pub(crate) fn flag(fields: &HashMap<String, Value>, name: &str) -> Result<bool, String> {
    match fields.get(name) {
        None | Some(Value::Nil) => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        Some(other) => Err(format!(
            "http_serve: `{name}` must be bool, got {}",
            other.type_name()
        )),
    }
}

/// Read an optional positive count, defaulting to `default`.
///
/// # Errors
///
/// Returns an error when the value is not a positive integer. Zero is refused because a
/// stream that may emit nothing is almost always a mistake, and saying so is better than
/// serving an empty body.
pub(crate) fn count(
    fields: &HashMap<String, Value>,
    name: &str,
    default: i64,
) -> Result<i64, String> {
    match fields.get(name) {
        None | Some(Value::Nil) => Ok(default),
        Some(Value::Int(value)) if *value > 0 => Ok(*value),
        Some(Value::Int(value)) => Err(format!(
            "http_serve: `{name}` must be positive, got {value}"
        )),
        Some(other) => Err(format!(
            "http_serve: `{name}` must be int, got {}",
            other.type_name()
        )),
    }
}

/// Read the status, defaulting to 200.
///
/// # Errors
///
/// Returns an error when the status is not an integer in the HTTP range.
pub(crate) fn status(fields: &HashMap<String, Value>) -> Result<u16, String> {
    match fields.get("status") {
        None | Some(Value::Nil) => Ok(200),
        Some(Value::Int(code)) if (100..=599).contains(code) => Ok(*code as u16),
        Some(Value::Int(code)) => Err(format!("http_serve: status {code} is out of range")),
        Some(other) => Err(format!(
            "http_serve: status must be int, got {}",
            other.type_name()
        )),
    }
}
