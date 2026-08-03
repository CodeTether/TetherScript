//! Argument coercion for the store built-ins.
//!
//! Split from registration so the built-in list stays readable. Every failure
//! names the parameter and the actual type, matching the wording the
//! signed-cookie half uses (`"<builtin>: <param> must be <type>, got <type>"`).

use std::collections::HashMap;

use crate::value::Value;

/// Coerce an argument to a string, naming the parameter on mismatch.
///
/// # Errors
///
/// Returns an error naming `label` and the actual type when `value` is not a str.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Coerce an argument to an int, naming the parameter on mismatch.
///
/// # Errors
///
/// Returns an error naming `label` and the actual type when `value` is not an int.
pub(super) fn int_arg(value: &Value, label: &str) -> Result<i64, String> {
    match value {
        Value::Int(number) => Ok(*number),
        other => Err(format!("{label} must be int, got {}", other.type_name())),
    }
}

/// Clone a map argument out into an owned payload.
///
/// The clone is deliberate: the store must not alias a map the script still
/// holds, or a later mutation there would edit stored session state without any
/// `store_save` call and defeat revocation auditing.
///
/// # Errors
///
/// Returns an error naming `label` and the actual type when `value` is not a map.
/// `nil` is accepted as an empty payload, so a session with no data yet needs no
/// ceremony at the call site.
pub(super) fn map_arg(value: &Value, label: &str) -> Result<HashMap<String, Value>, String> {
    match value {
        Value::Nil => Ok(HashMap::new()),
        Value::Map(map) => Ok(map.borrow().clone()),
        other => Err(format!("{label} must be map, got {}", other.type_name())),
    }
}
