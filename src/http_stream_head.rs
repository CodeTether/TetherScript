//! The head of a streaming response.
//!
//! `Content-Length` is the load-bearing *absence*: a streaming server cannot know the body
//! length, so framing comes from either `Transfer-Encoding: chunked` or connection close.
//! Emitting a length here would make a client wait for bytes that never come.

use std::collections::HashMap;

use super::http_status::reason_phrase;
use crate::value::Value;

/// Render the status line and headers for a streaming response.
///
/// # Arguments
///
/// * `status` — Validated status code.
/// * `fields` — The response map, for caller-supplied headers.
/// * `chunked` — Whether to frame with chunked coding.
///
/// # Returns
///
/// The complete head including the blank separator line.
pub(crate) fn render(status: u16, fields: &HashMap<String, Value>, chunked: bool) -> String {
    let mut head = format!("HTTP/1.1 {status} {}\r\n", reason_phrase(status));
    let supplied = headers_of(fields);

    if !supplied.iter().any(|(name, _)| name == "content-type") {
        // SSE's media type. A handler streaming something else overrides it.
        head.push_str("Content-Type: text/event-stream\r\n");
    }
    for (name, value) in &supplied {
        // A caller cannot set framing headers: they are decided by `chunked`, and a
        // conflicting value would desynchronize the body.
        if name == "content-length" || name == "transfer-encoding" || name == "connection" {
            continue;
        }
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    if chunked {
        head.push_str("Transfer-Encoding: chunked\r\nConnection: keep-alive\r\n");
    } else {
        // Without chunked coding the body's end *is* the connection close, so the
        // connection cannot be reused.
        head.push_str("Connection: close\r\n");
    }
    // Proxies buffer by default, which would defeat incremental delivery entirely.
    head.push_str("Cache-Control: no-cache\r\nX-Accel-Buffering: no\r\n\r\n");
    head
}

/// Extract caller-supplied headers, lowercased.
fn headers_of(fields: &HashMap<String, Value>) -> Vec<(String, String)> {
    let Some(Value::Map(headers)) = fields.get("headers") else {
        return Vec::new();
    };
    headers
        .borrow()
        .iter()
        .filter_map(|(name, value)| match value {
            Value::Str(text) => Some((name.to_ascii_lowercase(), (**text).clone())),
            _ => None,
        })
        .collect()
}
