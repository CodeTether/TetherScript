//! Field reads for the OAuth config map.
//!
//! Kept separate from the URL and body builders so every "missing or wrong-typed
//! config field" message is generated in exactly one place and reads the same way:
//! the built-in name, the field name, the expected type, and the type received.
//!
//! Absent and `nil` are treated alike, and so is the empty string. A script that
//! builds its config from optional request data naturally produces `""` for a value it
//! did not have, and treating that as "supplied" would emit `scope=` or, worse,
//! `state=` — an empty state being exactly the failure [`super`] exists to prevent.
//! Required fields therefore reject `""` explicitly.

use std::collections::HashMap;

use crate::value::Value;

/// Read a required string field from the config map.
///
/// # Arguments
///
/// * `config` — The script-supplied config map.
/// * `name` — Field name.
/// * `label` — Built-in name, used verbatim in the error message.
///
/// # Returns
///
/// The field's text, guaranteed non-empty.
///
/// # Errors
///
/// Returns `Err` when the field is absent, `nil`, empty, or not a string; the message
/// names the built-in and the field.
pub(crate) fn req_str(
    config: &HashMap<String, Value>,
    name: &str,
    label: &str,
) -> Result<String, String> {
    match config.get(name) {
        Some(Value::Str(text)) if !text.is_empty() => Ok((**text).clone()),
        Some(Value::Str(_)) => Err(format!("{label}: `{name}` must not be empty")),
        None | Some(Value::Nil) => Err(format!("{label}: config is missing `{name}`")),
        Some(other) => Err(format!(
            "{label}: `{name}` must be str, got {}",
            other.type_name()
        )),
    }
}

/// Read an optional string field, treating absent, `nil`, and `""` alike.
///
/// # Arguments
///
/// * `config` — The script-supplied config map.
/// * `name` — Field name.
/// * `label` — Built-in name, used verbatim in the error message.
///
/// # Returns
///
/// `Ok(None)` when the field is absent, `nil`, or the empty string; otherwise
/// `Ok(Some(text))`.
///
/// # Errors
///
/// Returns `Err` when the field is present but not a string.
pub(crate) fn opt_str(
    config: &HashMap<String, Value>,
    name: &str,
    label: &str,
) -> Result<Option<String>, String> {
    match config.get(name) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Str(text)) if text.is_empty() => Ok(None),
        Some(Value::Str(text)) => Ok(Some((**text).clone())),
        Some(other) => Err(format!(
            "{label}: `{name}` must be str, got {}",
            other.type_name()
        )),
    }
}
