//! Script item rendering for terminal rows.

use std::collections::HashMap;

use crate::value::Value;

use super::{style, style::Style, style_span};

pub(super) fn render(value: &Value) -> String {
    match value {
        Value::Map(map) => render_map(value, &map.borrow()),
        Value::Str(text) => text.to_string(),
        other => other.to_string(),
    }
}

fn render_map(value: &Value, map: &HashMap<String, Value>) -> String {
    if styled(map) {
        return styled_text(value, map).unwrap_or_else(|_| fallback(map));
    }
    fallback(map)
}

fn styled_text(value: &Value, map: &HashMap<String, Value>) -> Result<String, String> {
    if map.contains_key("kind") || map.contains_key("name") {
        let style = match map.get("style") {
            Some(value) => Style::parse(value)?,
            None => Style::from_fields(map)?,
        };
        return style::paint(&fallback(map), &style);
    }
    style_span::render(value)
}

fn styled(map: &HashMap<String, Value>) -> bool {
    map.contains_key("text")
        && ["style", "fg", "bg", "bold", "dim", "underline", "inverse"]
            .iter()
            .any(|key| map.contains_key(*key))
}

fn fallback(map: &HashMap<String, Value>) -> String {
    let kind = field(map, "kind");
    let name = field(map, "name");
    let text = field(map, "text");
    format!("[{kind}] {name}: {text}")
}

fn field(map: &HashMap<String, Value>, key: &str) -> String {
    map.get(key)
        .map(Value::to_string)
        .unwrap_or_else(|| "".into())
}
