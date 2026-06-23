//! OpenAI Responses-compatible provider adapter.

#[path = "provider_responses/body.rs"]
mod body;
#[path = "provider_responses/messages.rs"]
mod messages;
#[path = "provider_responses/model.rs"]
mod model;
#[path = "provider_responses/sse.rs"]
mod sse;

pub(crate) fn is_path(path: &str) -> bool {
    path.ends_with("/responses")
}

pub(crate) fn body(args: &[crate::value::Value], max_tokens: u64) -> Result<String, String> {
    body::build(args, max_tokens)
}

pub(crate) fn chat_json(text: &str) -> Result<crate::value::Value, String> {
    sse::chat_json(text)
}
