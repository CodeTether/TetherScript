//! Argument coercion and case-insensitive header lookup for the identity group.
//!
//! # Why this restates a rule instead of calling it
//!
//! `header_lookup::find` and `header_client_ip::resolve` already implement
//! case-insensitive lookup and proxy-address precedence, and this group composes
//! those *rules* rather than inventing new ones. It cannot call the functions: both
//! are `pub(super)` relative to the `header` module, so they are visible only to
//! `header`'s own children, and `header.rs` is owned by another concern and may not
//! be edited to widen them. The precedence encoded in
//! [`super::identity_context_fields::client_ip`] is therefore deliberately identical
//! to `header_client_ip`'s, and any change there must be mirrored.
//!
//! Lookup is case-insensitive because HTTP header names are (RFC 9110 §5.1). The
//! native parser lower-cases what it stores, but a script may build a header map by
//! hand from a fixture, so nothing here assumes normalisation.

use std::collections::HashMap;

use crate::value::Value;

/// Clone a map out of a script value, naming the parameter on mismatch.
///
/// # Arguments
///
/// * `value` — The candidate map.
/// * `label` — Parameter name to name in the error.
///
/// # Returns
///
/// A snapshot of the map's entries. A snapshot rather than a borrow so no long-lived
/// `RefCell` borrow can alias a map the script still holds.
///
/// # Errors
///
/// Returns an error naming the actual type when `value` is not a map.
pub(super) fn as_map(value: &Value, label: &str) -> Result<HashMap<String, Value>, String> {
    match value {
        Value::Map(map) => Ok(map.borrow().clone()),
        other => Err(format!("{label} must be a map, got {}", other.type_name())),
    }
}

/// Read a required string field out of a map.
///
/// # Arguments
///
/// * `map` — Map to read.
/// * `field` — Field name required to be present and a str.
/// * `label` — Built-in name, prefixed onto the error.
///
/// # Returns
///
/// The field's value.
///
/// # Errors
///
/// Returns an error naming `field` when it is absent or not a str.
pub(super) fn str_field(
    map: &HashMap<String, Value>,
    field: &str,
    label: &str,
) -> Result<String, String> {
    match map.get(field) {
        Some(Value::Str(text)) => Ok((**text).clone()),
        Some(other) => Err(format!(
            "{label}: `{field}` must be str, got {}",
            other.type_name()
        )),
        None => Err(format!("{label}: request map is missing `{field}`")),
    }
}

/// Coerce a built-in argument to a str, naming the parameter on mismatch.
///
/// # Errors
///
/// Returns an error naming the actual type when `value` is not a str.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Find a header value without regard to name casing.
///
/// # Arguments
///
/// * `headers` — Header map.
/// * `name` — Header name in any casing.
///
/// # Returns
///
/// The trimmed value, or `None` when no name matches or the value is not a str.
pub(super) fn find(headers: &HashMap<String, Value>, name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .and_then(|(_, value)| match value {
            Value::Str(text) => Some(text.trim().to_string()),
            _ => None,
        })
}
