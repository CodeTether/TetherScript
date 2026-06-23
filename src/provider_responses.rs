//! OpenAI Responses-compatible provider adapter.

#[path = "provider_responses/body.rs"]
mod body;
#[path = "provider_responses/messages.rs"]
mod messages;
#[path = "provider_responses/model.rs"]
mod model;
#[path = "provider_responses/output_call.rs"]
mod output_call;
#[path = "provider_responses/output_fields.rs"]
mod output_fields;
#[path = "provider_responses/output_items.rs"]
mod output_items;
#[path = "provider_responses/output_json.rs"]
mod output_json;
#[path = "provider_responses/output_shape.rs"]
mod output_shape;
#[path = "provider_responses/output_text.rs"]
mod output_text;
#[path = "provider_responses/sse.rs"]
mod sse;
#[path = "provider_responses/sse_event.rs"]
mod sse_event;
#[cfg(test)]
#[path = "provider_responses/tests.rs"]
mod tests;
#[path = "provider_responses/trace.rs"]
mod trace;

pub(crate) fn is_path(path: &str) -> bool {
    path.ends_with("/responses")
}

pub(crate) fn body(args: &[crate::value::Value], max_tokens: u64) -> Result<String, String> {
    body::build(args, max_tokens)
}

pub(crate) fn chat_json(text: &str) -> Result<crate::value::Value, String> {
    match output_json::chat_json(text) {
        Ok(value) => Ok(value),
        Err(_) if looks_like_sse(text) => sse::chat_json(text),
        Err(error) => Err(error),
    }
}

fn looks_like_sse(text: &str) -> bool {
    text.lines()
        .any(|line| line.trim_start().starts_with("data:"))
}
