//! Vault KV v2 provider listing parser.

use crate::value::Value;

pub(super) fn provider_ids(root: &Value) -> Result<Vec<String>, String> {
    let data = field(root, "data")?;
    let keys = field(&data, "keys")?;
    let Value::List(keys) = keys else {
        return Err("vault: data.keys must be a list".into());
    };
    let ids = keys
        .borrow()
        .iter()
        .filter_map(key_text)
        .collect::<Vec<_>>();
    Ok(ids)
}

fn key_text(value: &Value) -> Option<String> {
    match value {
        Value::Str(text) => Some(text.trim_end_matches('/').to_string()),
        _ => None,
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
