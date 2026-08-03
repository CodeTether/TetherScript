//! Argument coercion shared by the dynamic-page built-ins.
//!
//! Every coercion names the built-in and the parameter it was reading, so a
//! script author sees `page_cache_key: `slug` must be str, got int` rather than a
//! bare type error with no clue which field was wrong.

use std::collections::HashMap;

use crate::value::Value;

/// Read a str argument.
///
/// # Arguments
///
/// * `value` — Argument to read.
/// * `label` — Built-in and parameter name used in the error message.
///
/// # Returns
///
/// An owned copy of the string.
///
/// # Errors
///
/// Returns an error naming `label` and the received type when `value` is not a
/// str.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Clone a map argument out of a script value.
///
/// # Arguments
///
/// * `value` — Argument to read.
/// * `label` — Built-in and parameter name used in the error message.
///
/// # Returns
///
/// A snapshot of the map. The clone is deliberate: nothing in this group mutates a
/// caller's map, so no aliasing borrow is ever held across a call.
///
/// # Errors
///
/// Returns an error naming `label` and the received type when `value` is not a
/// map.
pub(super) fn map_arg(value: &Value, label: &str) -> Result<HashMap<String, Value>, String> {
    match value {
        Value::Map(map) => Ok(map.borrow().clone()),
        other => Err(format!("{label} must be a map, got {}", other.type_name())),
    }
}

/// Read a non-empty list of strings.
///
/// # Arguments
///
/// * `value` — Argument to read.
/// * `label` — Built-in and parameter name used in the error message.
///
/// # Returns
///
/// The strings in the caller's order, which is load-bearing for `locale_of`:
/// the first element is the default.
///
/// # Errors
///
/// Returns an error when `value` is not a list, when it is empty, or when any
/// element is not a str. An empty list is refused rather than defaulted, because
/// there would be no locale to fall back to and inventing one would return a
/// value the caller never declared.
pub(super) fn str_list_arg(value: &Value, label: &str) -> Result<Vec<String>, String> {
    let Value::List(items) = value else {
        return Err(format!("{label} must be a list, got {}", value.type_name()));
    };
    let items = items.borrow();
    if items.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    items
        .iter()
        .map(|item| str_arg(item, &format!("{label} entry")))
        .collect()
}
