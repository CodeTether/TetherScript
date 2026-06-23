//! Non-streaming Responses JSON conversion.

use crate::value::Value;

use super::{output_items, output_shape};

pub(super) fn chat_json(text: &str) -> Result<Value, String> {
    let parsed = crate::json::parse_str(text)
        .map_err(|error| format!("provider.responses: invalid response JSON: {error}"))?;
    if output_items::has_choices(&parsed) {
        return Ok(parsed);
    }
    let (content, tool_calls) = output_items::collect(&parsed)?;
    Ok(output_shape::chat(content, tool_calls))
}
