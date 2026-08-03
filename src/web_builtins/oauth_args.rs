//! Argument coercion for the OAuth built-ins.
//!
//! Kept separate from [`super::install_pkce`] and [`super::install_request`] so those stay
//! registration lists, and so every type error reads the same way: the built-in name, the
//! parameter name, the expected type, and the type actually received.

use std::collections::HashMap;

use crate::value::Value;

/// Coerce an argument to a string, naming the parameter on mismatch.
///
/// # Arguments
///
/// * `value` — The argument as received.
/// * `label` — `"builtin_name: parameter"`, used verbatim in the message.
///
/// # Returns
///
/// The string contents.
///
/// # Errors
///
/// Returns `Err` naming `label` and the actual type when `value` is not a `Str`.
pub(crate) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Coerce an argument to an integer, naming the parameter on mismatch.
///
/// # Arguments
///
/// * `value` — The argument as received.
/// * `label` — `"builtin_name: parameter"`, used verbatim in the message.
///
/// # Returns
///
/// The integer value.
///
/// # Errors
///
/// Returns `Err` naming `label` and the actual type when `value` is not an `Int`.
pub(crate) fn int_arg(value: &Value, label: &str) -> Result<i64, String> {
    match value {
        Value::Int(int) => Ok(*int),
        other => Err(format!("{label} must be int, got {}", other.type_name())),
    }
}

/// Clone a map argument, naming the parameter on mismatch.
///
/// The map is cloned rather than borrowed so no `RefCell` borrow is held across the
/// validation and formatting that follows, which could otherwise panic if a script passed
/// the same map as two arguments.
///
/// # Arguments
///
/// * `value` — The argument as received.
/// * `label` — `"builtin_name: parameter"`, used verbatim in the message.
///
/// # Returns
///
/// An owned copy of the map's fields.
///
/// # Errors
///
/// Returns `Err` naming `label` and the actual type when `value` is not a `Map`.
pub(crate) fn map_arg(value: &Value, label: &str) -> Result<HashMap<String, Value>, String> {
    match value {
        Value::Map(fields) => Ok(fields.borrow().clone()),
        other => Err(format!("{label} must be map, got {}", other.type_name())),
    }
}
