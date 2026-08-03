//! Response-map and header-map construction for an event stream.
//!
//! # Why each header is mandatory, not advisory
//!
//! * `content-type: text/event-stream` — without it a browser `EventSource`
//!   rejects the response outright and fires `error`, and a fetch-based client
//!   gets a document it will try to render.
//! * `cache-control: no-store` — an event stream is a sequence of *events*, not a
//!   representation. If any cache stores it, every later reader is served the
//!   captured prefix forever: the stream appears to connect, replays stale
//!   events, then hangs. `no-store` is used rather than `no-cache` because
//!   `no-cache` still permits storage subject to revalidation, and a revalidated
//!   stream body is meaningless.
//! * `connection: keep-alive` — the transport must not close after the first
//!   write. This is declarative only; see `ssestream_spec` for what the
//!   server itself still has to do.
//! * `x-accel-buffering: no` — a reverse proxy that response-buffers will hold
//!   events until its buffer fills, turning a live stream into a batch delivery.
//!   The header is nginx-specific but harmless elsewhere.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Build the header map every event-stream response needs.
///
/// # Returns
///
/// A map value with lowercase header names, matching what
/// `http_response_extract` lowercases anyway, so a caller merging these into its
/// own headers cannot end up with two casings of the same header.
pub(super) fn headers() -> Value {
    let mut headers = HashMap::new();
    for (name, value) in [
        ("content-type", "text/event-stream; charset=utf-8"),
        ("cache-control", "no-store"),
        ("connection", "keep-alive"),
        ("x-accel-buffering", "no"),
    ] {
        headers.insert(name.to_string(), Value::Str(Rc::new(value.to_string())));
    }
    Value::Map(Rc::new(RefCell::new(headers)))
}

/// Build a complete `200` response map carrying an already-framed body.
///
/// # Arguments
///
/// * `body` — The full concatenated stream text. Pass the empty string for an
///   empty stream; that is a valid zero-event stream, not an error.
///
/// # Returns
///
/// A map with `status`, `headers`, and `body`, the shape `http_serve` already
/// consumes.
pub(super) fn response(body: String) -> Value {
    let mut response = HashMap::new();
    response.insert("status".into(), Value::Int(200));
    response.insert("headers".into(), headers());
    response.insert("body".into(), Value::Str(Rc::new(body)));
    Value::Map(Rc::new(RefCell::new(response)))
}
