//! Field readers for TUI style maps.

use std::collections::HashMap;

use crate::value::Value;

pub(super) fn bool_field(map: &HashMap<String, Value>, key: &str) -> Result<bool, String> {
    match map.get(key) {
        Some(Value::Bool(value)) => Ok(*value),
        Some(Value::Nil) | None => Ok(false),
        Some(other) => Err(format!(
            "tui_style: {key} must be bool, got {}",
            other.type_name()
        )),
    }
}

pub(super) fn text_field(
    map: &HashMap<String, Value>,
    key: &str,
) -> Result<Option<String>, String> {
    match map.get(key) {
        Some(Value::Str(value)) => Ok(Some(value.to_string())),
        Some(Value::Nil) | None => Ok(None),
        Some(other) => Err(format!(
            "tui_style: {key} must be str, got {}",
            other.type_name()
        )),
    }
}

pub(super) fn required_text(
    map: &HashMap<String, Value>,
    key: &str,
    label: &str,
) -> Result<String, String> {
    text_field(map, key)?.ok_or_else(|| format!("{label}: missing {key}"))
}
