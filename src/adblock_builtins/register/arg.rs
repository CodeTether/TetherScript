//! Argument extraction helpers.

use crate::value::Value;

/// Extract a string argument with a named error.
pub(super) fn str_arg(value: &Value, name: &str) -> Result<String, String> {
    match value {
        Value::Str(s) => Ok((**s).clone()),
        _ => Err(format!(
            "adblock: {name} must be str, got {}",
            value.type_name()
        )),
    }
}
