//! Responses message conversion.

use crate::value::Value;

use super::body;

#[path = "message_fields.rs"]
mod fields;
#[path = "message_items.rs"]
mod items;
#[path = "message_tools.rs"]
mod tools;

pub(super) fn instructions(messages: &Value) -> String {
    text_for_role(messages, "system").unwrap_or_else(|| {
        "You are CodeTether Agent running on OpenAI Codex. Reply concisely.".into()
    })
}

pub(super) fn input(messages: &Value) -> Result<String, String> {
    let mut items = Vec::new();
    if let Value::List(list) = messages {
        for item in list.borrow().iter() {
            push_item(&mut items, item)?;
        }
    }
    Ok(format!("[{}]", items.join(",")))
}

fn push_item(items: &mut Vec<String>, item: &Value) -> Result<(), String> {
    match fields::role(item).as_deref() {
        Some("user") => items::push_text(items, item, "user")?,
        Some("assistant") => {
            items::push_text(items, item, "assistant")?;
            tools::push_calls(items, item)?;
        }
        Some("tool") => tools::push_output(items, item)?,
        _ => {}
    }
    Ok(())
}

fn text_for_role(messages: &Value, expected: &str) -> Option<String> {
    let Value::List(list) = messages else {
        return None;
    };
    let parts = list
        .borrow()
        .iter()
        .filter(|item| fields::role(item).as_deref() == Some(expected))
        .filter_map(fields::content)
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("\n\n"))
}

pub(super) fn string(text: &str) -> Result<String, String> {
    body::string(text)
}
