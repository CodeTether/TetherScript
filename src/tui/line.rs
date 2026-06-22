//! ASCII line shaping for terminal frames.

use crate::value::Value;

pub(super) fn fit(text: &str, width: usize) -> String {
    let mut out: String = text.chars().take(width).collect();
    while out.len() < width {
        out.push(' ');
    }
    out
}

pub(super) fn item(value: &Value) -> String {
    match value {
        Value::Map(map) => {
            let map = map.borrow();
            let kind = field(&map, "kind");
            let name = field(&map, "name");
            let text = field(&map, "text");
            format!("[{kind}] {name}: {text}")
        }
        Value::Str(text) => text.to_string(),
        other => other.to_string(),
    }
}

fn field(map: &std::collections::HashMap<String, Value>, key: &str) -> String {
    map.get(key)
        .map(Value::to_string)
        .unwrap_or_else(|| "".into())
}
