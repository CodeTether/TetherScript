//! Assembly of one complete wire chunk: an event, a keepalive, or a retry.
//!
//! A chunk is the unit a streaming server writes and flushes. It is always
//! self-contained and always ends at a point where the client can act, so a
//! server may write chunks back-to-back with no framing state of its own.
//!
//! # Field order
//!
//! `event`, then `id`, then `retry`, then `data`. The order is fixed and
//! independent of map insertion order so captures are diffable and tests can pin
//! exact bytes. `data` is last because it is the only multi-line field: a
//! truncated capture is then obviously truncated.
//!
//! # The blank line is the dispatch signal
//!
//! A client buffers fields and only fires the event when it reads a blank line.
//! A frame missing its terminator is not a late event — it is an event that never
//! arrives, while the connection looks perfectly healthy.

use std::collections::HashMap;

use crate::value::Value;

use super::{ssestream_data as data, ssestream_field as field};

/// Frame one event from a map of fields.
///
/// # Arguments
///
/// * `fields` — Map that may carry `event`, `id`, `retry`, and `data`. Unknown
///   keys are ignored so a caller can pass a richer domain record straight
///   through. A map with no recognized key yields a bare `"\n"`, which is a
///   valid no-op the client discards.
///
/// # Returns
///
/// The exact bytes of one chunk, terminated by the dispatching blank line.
///
/// # Errors
///
/// Returns an error when `fields` is not a map, when `event` or `id` is not a
/// single-line str, when `id` contains NUL, or when `retry` is not a
/// non-negative int.
pub(super) fn event(fields: &Value) -> Result<String, String> {
    let Value::Map(map) = fields else {
        return Err(format!(
            "sse: event must be a map, got {}",
            fields.type_name()
        ));
    };
    render(&map.borrow())
}

/// Render an already-borrowed field map, so callers holding a borrow can reuse it.
fn render(map: &HashMap<String, Value>) -> Result<String, String> {
    let mut out = String::new();
    if let Some(value) = map.get("event") {
        out.push_str(&field::line("event", value)?);
    }
    if let Some(value) = map.get("id") {
        out.push_str(&field::id_line(value)?);
    }
    if let Some(value) = map.get("retry") {
        out.push_str(&field::retry_line(value)?);
    }
    if let Some(value) = map.get("data") {
        out.push_str(&data::lines(value));
    }
    out.push('\n');
    Ok(out)
}

/// Build the keepalive comment chunk.
///
/// # Returns
///
/// `": keepalive\n\n"`. A comment line starts with `:` and carries no fields, so
/// the client parses it, dispatches nothing, and resets its read timer. The
/// trailing blank line is included because an intermediary that buffers by line
/// boundary otherwise has no reason to forward the comment at all.
pub(super) fn keepalive() -> String {
    ": keepalive\n\n".to_string()
}

/// Build a bare `retry:` chunk.
///
/// # Arguments
///
/// * `ms` — Reconnection delay in whole milliseconds. Must be a non-negative int.
///
/// # Returns
///
/// `"retry: <ms>\n\n"`.
///
/// # Errors
///
/// Returns an error naming the field when `ms` is not a non-negative int.
pub(super) fn retry(ms: &Value) -> Result<String, String> {
    Ok(format!("{}\n", field::retry_line(ms)?))
}
