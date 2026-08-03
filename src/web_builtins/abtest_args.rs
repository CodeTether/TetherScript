//! Argument and field coercion for the A/B test group.
//!
//! Split out so every other module can report a type error that names the built-in,
//! the field, and the type actually received, rather than a bare "type error".

use std::collections::HashMap;

use crate::value::Value;

/// Borrow a `Value` as a str.
///
/// # Arguments
///
/// * `value` — The script value to read.
/// * `label` — Already-qualified description, e.g. `"ab_assign: subject"`.
///
/// # Returns
///
/// An owned copy of the string.
///
/// # Errors
///
/// Returns an error naming `label` and the received type when `value` is not a str.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Borrow a `Value` as a non-empty str.
///
/// # Errors
///
/// Returns an error when `value` is not a str, or when it is empty. An empty seed
/// or subject would collapse every visitor onto one bucket, which is a silent way
/// to break an experiment, so it is refused rather than hashed.
pub(super) fn nonempty_str(value: &Value, label: &str) -> Result<String, String> {
    let text = str_arg(value, label)?;
    if text.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    Ok(text)
}

/// Clone a `Value` out as a map.
///
/// # Errors
///
/// Returns an error naming `label` and the received type when `value` is not a map.
pub(super) fn map_arg(value: &Value, label: &str) -> Result<HashMap<String, Value>, String> {
    match value {
        Value::Map(entries) => Ok(entries.borrow().clone()),
        other => Err(format!("{label} must be a map, got {}", other.type_name())),
    }
}

/// Read a required str field from a map.
///
/// # Errors
///
/// Returns an error when the key is absent or nil, when it is not a str, or when
/// it is empty.
pub(super) fn field_str(
    entries: &HashMap<String, Value>,
    key: &str,
    label: &str,
) -> Result<String, String> {
    match entries.get(key) {
        None | Some(Value::Nil) => Err(format!("{label}: missing `{key}`")),
        Some(value) => nonempty_str(value, &format!("{label}: `{key}`")),
    }
}

/// Read an optional str field from a map.
///
/// # Returns
///
/// `Ok(None)` when the key is absent, nil, or the empty string — an empty cookie
/// name is indistinguishable from "no sticky cookie configured", so both mean the
/// same thing rather than producing a cookie with no name.
///
/// # Errors
///
/// Returns an error when the value is present but not a str.
pub(super) fn field_opt_str(
    entries: &HashMap<String, Value>,
    key: &str,
    label: &str,
) -> Result<Option<String>, String> {
    match entries.get(key) {
        None | Some(Value::Nil) => Ok(None),
        Some(value) => {
            let text = str_arg(value, &format!("{label}: `{key}`"))?;
            Ok((!text.is_empty()).then_some(text))
        }
    }
}
