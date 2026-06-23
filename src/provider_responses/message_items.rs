use crate::value::Value;

use super::{fields, string};

pub(super) fn push_text(items: &mut Vec<String>, item: &Value, role: &str) -> Result<(), String> {
    let Some(text) = fields::content(item) else {
        return Ok(());
    };
    if text.is_empty() {
        return Ok(());
    }
    items.push(format!(
        "{{\"type\":\"message\",\"role\":{},\"content\":[{}]}}",
        string(role)?,
        text_part(role, &text)?
    ));
    Ok(())
}

fn text_part(role: &str, text: &str) -> Result<String, String> {
    let kind = if role == "assistant" {
        "output_text"
    } else {
        "input_text"
    };
    Ok(format!(
        "{{\"type\":{},\"text\":{}}}",
        string(kind)?,
        string(text)?
    ))
}
