//! Typed extraction from the `cookie_serialize` options map.
//!
//! Options are optional by design, so a missing key yields `None` rather than an
//! error. A key that is present but the wrong type *is* an error: silently
//! ignoring a mistyped option would drop a security attribute such as `HttpOnly`
//! without telling the caller.

use std::collections::HashMap;

use super::cookie_alias::lookup;
use crate::value::Value;

/// Read an optional string attribute such as `Path` or `Domain`.
///
/// # Arguments
///
/// * `opts` — The script-supplied options map.
/// * `key` — Snake-case option name; header casing is also accepted.
///
/// # Returns
///
/// `Ok(None)` when absent or nil, otherwise the string.
///
/// # Errors
///
/// Returns an error naming `key` when the value is present but not a str.
pub(super) fn string(opts: &HashMap<String, Value>, key: &str) -> Result<Option<String>, String> {
    match lookup(opts, key) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Str(text)) => Ok(Some(text.to_string())),
        Some(other) => Err(type_error(key, "str", other)),
    }
}

/// Read an optional integer attribute, such as `Max-Age` in seconds.
///
/// # Errors
///
/// Returns an error naming `key` when the value is present but not an int.
pub(super) fn integer(opts: &HashMap<String, Value>, key: &str) -> Result<Option<i64>, String> {
    match lookup(opts, key) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Int(int)) => Ok(Some(*int)),
        Some(other) => Err(type_error(key, "int", other)),
    }
}

/// Read a boolean flag such as `HttpOnly` or `Secure`, defaulting to false.
///
/// # Errors
///
/// Returns an error naming `key` when the value is present but not a bool.
pub(super) fn flag(opts: &HashMap<String, Value>, key: &str) -> Result<bool, String> {
    match lookup(opts, key) {
        None | Some(Value::Nil) => Ok(false),
        Some(Value::Bool(flag)) => Ok(*flag),
        Some(other) => Err(type_error(key, "bool", other)),
    }
}

fn type_error(key: &str, expected: &str, actual: &Value) -> String {
    format!(
        "cookie option `{key}` must be {expected}, got {}",
        actual.type_name()
    )
}
