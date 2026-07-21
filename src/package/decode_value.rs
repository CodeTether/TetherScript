//! Primitive manifest field decoding and validation.

use std::collections::HashMap;

use crate::value::Value;

pub(super) fn object<'a>(
    value: &'a Value,
    field: &str,
) -> Result<std::cell::Ref<'a, HashMap<String, Value>>, String> {
    match value {
        Value::Map(map) => Ok(map.borrow()),
        other => Err(format!(
            "{field} must be an object, got {}",
            other.type_name()
        )),
    }
}

pub(super) fn string(map: &HashMap<String, Value>, key: &str) -> Result<String, String> {
    match map.get(key) {
        Some(Value::Str(value)) if !value.is_empty() => Ok(value.as_ref().clone()),
        Some(Value::Str(_)) => Err(format!("package.{key} must not be empty")),
        Some(other) => Err(format!(
            "package.{key} must be a string, got {}",
            other.type_name()
        )),
        None => Err(format!("missing package.{key}")),
    }
}

pub(super) fn reject_keys(
    map: &HashMap<String, Value>,
    allowed: &[&str],
    field: &str,
) -> Result<(), String> {
    map.keys()
        .find(|key| !allowed.contains(&key.as_str()))
        .map_or(Ok(()), |key| Err(format!("unknown {field} field `{key}`")))
}
