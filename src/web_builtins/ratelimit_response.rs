//! `Retry-After` and the 429 response map.
//!
//! The response map matches `examples/the reference application/server/response.tether`
//! exactly — `status`, `headers`, `body` — because that is the shape `http_serve`
//! already consumes.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Convert milliseconds to the whole seconds a `Retry-After` header carries.
///
/// # Arguments
///
/// * `retry_after_ms` — Milliseconds the caller must wait.
///
/// # Returns
///
/// Seconds, always rounded **up**, and at least 1 whenever any wait is required.
/// Rounding down would tell the client to retry while it is still limited, which
/// produces a retry storm precisely when the server is already overloaded.
pub(super) fn header_seconds(retry_after_ms: i64) -> i64 {
    if retry_after_ms <= 0 {
        return 0;
    }
    // Integer ceiling division: avoids the float rounding a /1000.0 would add.
    (retry_after_ms + 999) / 1000
}

/// Build the 429 response, carrying a `Retry-After` header.
///
/// # Arguments
///
/// * `retry_after_ms` — Milliseconds until the caller may retry.
///
/// # Returns
///
/// A response map with status 429, a `retry-after` header in seconds, and a plain
/// text body. Header names are lowercase to match the request map built by
/// `src/http_server_request_map.rs`.
pub(super) fn too_many_requests(retry_after_ms: i64) -> Value {
    let seconds = header_seconds(retry_after_ms);
    let mut headers = HashMap::new();
    headers.insert("retry-after".into(), Value::Int(seconds));
    headers.insert(
        "content-type".into(),
        Value::Str(Rc::new("text/plain; charset=utf-8".into())),
    );

    let mut response = HashMap::new();
    response.insert("status".into(), Value::Int(429));
    response.insert("headers".into(), Value::Map(Rc::new(RefCell::new(headers))));
    response.insert(
        "body".into(),
        Value::Str(Rc::new(format!(
            "too many requests; retry after {seconds}s\n"
        ))),
    );
    Value::Map(Rc::new(RefCell::new(response)))
}
