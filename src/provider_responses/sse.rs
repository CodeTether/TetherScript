//! Responses SSE parsing.

use crate::value::Value;

use super::{output_items, output_shape, sse_event};

pub(super) fn chat_json(text: &str) -> Result<Value, String> {
    let mut state = sse_event::State::new();
    for line in text.lines() {
        let Some(data) = data_line(line) else {
            continue;
        };
        if data == "[DONE]" {
            break;
        }
        let event = crate::json::parse_str(data)
            .map_err(|error| format!("provider.responses: invalid SSE JSON: {error}"))?;
        state.apply(&event)?;
    }
    if let Some(response) = state.response {
        let (text, calls) = output_items::collect(&response)?;
        if !text.is_empty() || !calls.is_empty() {
            return Ok(output_shape::chat(text, calls));
        }
    }
    Ok(output_shape::chat(state.text, state.calls))
}

fn data_line(line: &str) -> Option<&str> {
    line.trim_start().strip_prefix("data:").map(str::trim_start)
}
