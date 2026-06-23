//! Responses output item collection.

use crate::value::Value;

use super::{output_call, output_fields as fields, output_text};

pub(super) fn has_choices(value: &Value) -> bool {
    fields::field(value, "choices").is_some()
}

pub(super) fn collect(root: &Value) -> Result<(String, Vec<Value>), String> {
    let output =
        fields::list(root, "output").ok_or("provider.responses: response missing output list")?;
    let mut text = Vec::new();
    let mut calls = Vec::new();
    for item in output.borrow().iter() {
        match fields::string(item, "type").as_deref() {
            Some("message") if assistant(item) => text.extend(output_text::parts(item)),
            Some("function_call") => calls.push(output_call::tool_call(item)?),
            _ => {}
        }
    }
    if text.is_empty() {
        if let Some(output_text) = fields::string(root, "output_text") {
            text.push(output_text);
        }
    }
    Ok((text.join(""), calls))
}

fn assistant(item: &Value) -> bool {
    match fields::string(item, "role") {
        Some(role) => role == "assistant",
        None => true,
    }
}
