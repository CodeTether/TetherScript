//! HAR response field serialization.

use crate::browser_agent::BrowserHarResponse;
use crate::value::Value;

#[path = "diagnose_har_headers.rs"]
mod headers;

pub(super) fn value(response: BrowserHarResponse) -> Value {
    super::super::super::super::value::map(vec![
        ("status", Value::Int(response.status.into())),
        (
            "status_text",
            super::super::super::super::value::string(response.status_text),
        ),
        ("headers", headers::value(response.headers)),
        ("content_text", optional(response.content_text)),
        ("route_result", optional(response.route_result)),
    ])
}

fn optional(value: Option<String>) -> Value {
    value.map_or(Value::Nil, super::super::super::super::value::string)
}
