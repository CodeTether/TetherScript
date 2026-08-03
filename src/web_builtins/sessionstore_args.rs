//! Argument coercion for the session-store built-ins.
//!
//! Split from registration so the built-in list stays one readable line per entry.
//! Every failure names the parameter and the actual type, matching the wording the
//! signed-cookie group uses: `"<builtin>: <param> must be <type>, got <type>"`.
//! A silent coercion here would be dangerous: a `limit` that defaulted to zero on a
//! bad argument turns into a limiter that denies everything.

use std::collections::HashMap;

use crate::value::Value;

/// Coerce an argument to a string.
///
/// # Arguments
///
/// * `value` — The argument.
/// * `label` — `"<builtin>: <param>"`, used verbatim in the error.
///
/// # Returns
///
/// An owned copy of the string.
///
/// # Errors
///
/// Returns a named error when `value` is not a str.
///
/// # Examples
///
/// ```rust,ignore
/// assert!(str_arg(&crate::value::Value::Nil, "f: a").is_err());
/// ```
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Coerce an argument to an int.
///
/// # Arguments
///
/// * `value` — The argument.
/// * `label` — `"<builtin>: <param>"`, used verbatim in the error.
///
/// # Returns
///
/// The integer. Floats are rejected rather than truncated: a truncated window or
/// limit is a different policy than the one the caller wrote.
///
/// # Errors
///
/// Returns a named error when `value` is not an int.
pub(super) fn int_arg(value: &Value, label: &str) -> Result<i64, String> {
    match value {
        Value::Int(number) => Ok(*number),
        other => Err(format!("{label} must be int, got {}", other.type_name())),
    }
}

/// Clone a map argument out into an owned payload.
///
/// The clone is deliberate: serialization must not alias a map the script still
/// holds, and it keeps this group free of any borrow the interpreter could observe.
///
/// # Errors
///
/// Returns a named error when `value` is not a map.
pub(super) fn map_arg(value: &Value, label: &str) -> Result<HashMap<String, Value>, String> {
    match value {
        Value::Map(entries) => Ok(entries.borrow().clone()),
        other => Err(format!("{label} must be a map, got {}", other.type_name())),
    }
}
