//! Argument coercion shared by the CORS built-ins.
//!
//! Every coercion names the built-in and the parameter it was reading, so a
//! script author sees `cors_policy: `origins` must be a list, got int` rather
//! than a bare type error with no idea which field was wrong.

use std::collections::HashMap;

use crate::value::Value;

/// Clone a map argument out of a script value.
///
/// # Arguments
///
/// * `value` — The argument to read.
/// * `label` — Built-in and parameter name used in the error message.
///
/// # Returns
///
/// A snapshot of the map. The clone is deliberate: nothing in this group mutates
/// a caller's map, so no aliasing borrow is ever held.
///
/// # Errors
///
/// Returns an error naming the actual type when `value` is not a map.
pub(super) fn map_arg(value: &Value, label: &str) -> Result<HashMap<String, Value>, String> {
    match value {
        Value::Map(map) => Ok(map.borrow().clone()),
        other => Err(format!("{label} must be a map, got {}", other.type_name())),
    }
}

/// Read a string argument.
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

/// Read a list of strings.
///
/// # Errors
///
/// Returns an error when `value` is not a list, or when any entry is not a str.
pub(super) fn string_list(value: &Value, label: &str) -> Result<Vec<String>, String> {
    let Value::List(items) = value else {
        return Err(format!("{label} must be a list, got {}", value.type_name()));
    };
    items
        .borrow()
        .iter()
        .map(|item| str_arg(item, &format!("{label} entry")))
        .collect()
}
