//! Argument handling for the streaming built-ins.
//!
//! Each function here is the thin adapter between a built-in's argument slice and
//! the pure formatting modules, so registration stays a flat list and the
//! formatting code never sees a `&[Value]`.

use std::rc::Rc;

use crate::value::Value;

use super::{
    ssestream_batch as batch, ssestream_chunk as chunk, ssestream_response as response_map,
};

/// `sse_stream_response(events)` — batch response map.
///
/// # Arguments
///
/// * `args` — One element: the list of event field maps.
///
/// # Returns
///
/// A response map with status `200`, the event-stream headers, and the framed
/// body.
///
/// # Errors
///
/// Returns an error when the argument is not a list, or when any event is
/// malformed; the message carries the failing event's index.
pub(super) fn stream_response(args: &[Value]) -> Result<Value, String> {
    Ok(response_map::response(batch::body(&args[0])?))
}

/// `sse_stream_headers()` — the header map alone, for a hand-built response.
///
/// # Returns
///
/// A map of lowercase header names to values.
pub(super) fn stream_headers() -> Value {
    response_map::headers()
}

/// `sse_chunk(event)` — exact wire bytes of one event.
///
/// # Arguments
///
/// * `args` — One element: the event field map.
///
/// # Returns
///
/// A str holding one complete, blank-line-terminated chunk.
///
/// # Errors
///
/// Returns an error when the argument is not a map, or a field is invalid.
pub(super) fn stream_chunk(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Str(Rc::new(chunk::event(&args[0])?)))
}

/// `sse_keepalive()` — the comment-only chunk.
///
/// # Returns
///
/// A str holding `": keepalive\n\n"`. Never fails, so it is not a `Result`.
pub(super) fn keepalive() -> Value {
    Value::Str(Rc::new(chunk::keepalive()))
}

/// `sse_retry_frame(ms)` — the retry directive chunk.
///
/// # Arguments
///
/// * `args` — One element: the delay in milliseconds.
///
/// # Returns
///
/// A str holding `"retry: <ms>\n\n"`.
///
/// # Errors
///
/// Returns an error when `ms` is not a non-negative int.
pub(super) fn retry_frame(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Str(Rc::new(chunk::retry(&args[0])?)))
}
