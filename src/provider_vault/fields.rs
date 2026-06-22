//! Extract fields from Vault JSON values.

use std::collections::HashMap;

use crate::value::Value;

pub(super) type Map = HashMap<String, Value>;

pub(super) fn kv2_secret(root: &Value) -> Result<Value, String> {
    let data = field(root, "data")?;
    field(&data, "data")
}

pub(super) fn string(map: &Map, key: &str) -> Option<String> {
    match map.get(key) {
        Some(Value::Str(value)) if !value.is_empty() => Some(value.to_string()),
        _ => None,
    }
}

pub(super) fn headers(map: &Map) -> Vec<(String, String)> {
    match map.get("headers") {
        Some(Value::Map(headers)) => headers
            .borrow()
            .iter()
            .filter_map(|(name, value)| match value {
                Value::Str(text) => Some((name.clone(), text.to_string())),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn field(value: &Value, key: &str) -> Result<Value, String> {
    match value {
        Value::Map(map) => map
            .borrow()
            .get(key)
            .cloned()
            .ok_or_else(|| format!("vault: missing {key} field")),
        other => Err(format!("vault: expected map, got {}", other.type_name())),
    }
}
