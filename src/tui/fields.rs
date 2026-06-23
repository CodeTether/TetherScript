//! Shared field readers for structured terminal view maps.

use std::collections::HashMap;

use crate::value::Value;

use super::val;

pub(super) fn text(map: &HashMap<String, Value>, key: &str) -> String {
    match map.get(key) {
        Some(Value::Str(text)) => text.to_string(),
        Some(value) => value.to_string(),
        None => String::new(),
    }
}

pub(super) fn usize_or(
    map: &HashMap<String, Value>,
    key: &str,
    default: usize,
    min: i64,
    max: i64,
) -> Result<usize, String> {
    match map.get(key) {
        Some(value) => Ok(val::int_arg(value, key)?.clamp(min, max) as usize),
        None => Ok(default),
    }
}

pub(super) fn maybe_usize(
    map: &HashMap<String, Value>,
    key: &str,
    min: i64,
    max: i64,
) -> Result<Option<usize>, String> {
    map.get(key)
        .map(|value| val::int_arg(value, key).map(|n| n.clamp(min, max) as usize))
        .transpose()
}

pub(super) fn list(
    map: &HashMap<String, Value>,
    key: &str,
    label: &str,
) -> Result<Vec<Value>, String> {
    match map.get(key) {
        Some(Value::List(items)) => Ok(items.borrow().clone()),
        Some(other) => Err(format!("{label} must be list, got {}", other.type_name())),
        None => Ok(Vec::new()),
    }
}
