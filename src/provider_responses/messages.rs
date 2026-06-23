//! Responses message conversion.

use crate::value::Value;

use super::body;

pub(super) fn instructions(messages: &Value) -> String {
    text_for_role(messages, "system").unwrap_or_else(|| {
        "You are CodeTether Agent running on OpenAI Codex. Reply concisely.".into()
    })
}

pub(super) fn input(messages: &Value) -> Result<String, String> {
    let mut items = Vec::new();
    if let Value::List(list) = messages {
        for item in list.borrow().iter() {
            if role(item).as_deref() == Some("user") {
                if let Some(text) = content(item) {
                    items.push(format!(
                        "{{\"type\":\"message\",\"role\":\"user\",\"content\":[{{\"type\":\"input_text\",\"text\":{}}}]}}",
                        body::string(&text)?
                    ));
                }
            }
        }
    }
    Ok(format!("[{}]", items.join(",")))
}

fn text_for_role(messages: &Value, expected: &str) -> Option<String> {
    let Value::List(list) = messages else {
        return None;
    };
    let parts = list
        .borrow()
        .iter()
        .filter(|item| role(item).as_deref() == Some(expected))
        .filter_map(content)
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("\n\n"))
}

fn role(value: &Value) -> Option<String> {
    field(value, "role")
}

fn content(value: &Value) -> Option<String> {
    field(value, "content")
}

fn field(value: &Value, key: &str) -> Option<String> {
    let Value::Map(map) = value else {
        return None;
    };
    match map.borrow().get(key) {
        Some(Value::Str(text)) => Some(text.to_string()),
        _ => None,
    }
}
