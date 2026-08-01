//! Case-insensitive header lookup.
//!
//! HTTP header names are case-insensitive (RFC 9110 §5.1), so a lookup that only
//! matches exact case silently misses a real header. The native parser in
//! `crate::http_server_headers` already lower-cases every name it stores, but a
//! script may build a header map by hand — from a test fixture or an upstream
//! response — so this module never assumes the map was normalized.

use std::collections::HashMap;

use crate::value::Value;

/// Borrow the header map out of a script value.
///
/// # Arguments
///
/// * `value` — The `headers` map a handler received.
///
/// # Errors
///
/// Returns an error naming the actual type when `value` is not a map.
pub(super) fn as_map(value: &Value, label: &str) -> Result<HashMap<String, Value>, String> {
    match value {
        Value::Map(map) => Ok(map.borrow().clone()),
        other => Err(format!(
            "{label} must be a headers map, got {}",
            other.type_name()
        )),
    }
}

/// Find a header value, comparing names without regard to case.
///
/// # Arguments
///
/// * `headers` — Header map.
/// * `name` — Header name in any casing.
///
/// # Returns
///
/// The trimmed value, or `None` when no name matches.
pub(super) fn find(headers: &HashMap<String, Value>, name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .and_then(|(_, value)| match value {
            Value::Str(text) => Some(text.trim().to_string()),
            _ => None,
        })
}
