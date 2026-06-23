//! Responses SSE parsing.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub(super) fn chat_json(text: &str) -> Result<Value, String> {
    let content = output_text(text)?;
    let mut msg = HashMap::new();
    msg.insert("role".into(), Value::Str(Rc::new("assistant".into())));
    msg.insert("content".into(), Value::Str(Rc::new(content)));
    let mut choice = HashMap::new();
    choice.insert("message".into(), Value::Map(Rc::new(RefCell::new(msg))));
    let mut root = HashMap::new();
    root.insert(
        "choices".into(),
        Value::List(Rc::new(RefCell::new(vec![Value::Map(Rc::new(
            RefCell::new(choice),
        ))]))),
    );
    Ok(Value::Map(Rc::new(RefCell::new(root))))
}

fn output_text(text: &str) -> Result<String, String> {
    let mut out = String::new();
    for line in text.lines() {
        let Some(data) = line.strip_prefix("data: ") else {
            continue;
        };
        let event = crate::json::parse_str(data)
            .map_err(|error| format!("provider.responses: invalid SSE JSON: {error}"))?;
        if event_type(&event).as_deref() == Some("response.output_text.delta") {
            if let Some(delta) = string_field(&event, "delta") {
                out.push_str(&delta);
            }
        }
    }
    Ok(out)
}

fn event_type(value: &Value) -> Option<String> {
    string_field(value, "type")
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    let Value::Map(map) = value else {
        return None;
    };
    match map.borrow().get(key) {
        Some(Value::Str(text)) => Some(text.to_string()),
        _ => None,
    }
}
