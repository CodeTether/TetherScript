//! Streaming responses for `http_serve`.
//!
//! An ordinary response is written once and the connection closes or is reused. A
//! *streaming* response instead pulls events from a script generator and writes each one as
//! it is produced, which is what SSE requires: a client must see the first event while
//! later ones are still being computed.
//!
//! A handler opts in by returning a map with a `stream` field holding a callable. Optional
//! `chunked` selects `Transfer-Encoding: chunked` over connection-close framing, and
//! `max_events` bounds a generator that never finishes.

use std::io::Write;

use crate::value::{Runtime, Value};

/// Whether `resp` asked for a streaming body.
///
/// # Arguments
///
/// * `resp` — The value a handler returned.
///
/// # Returns
///
/// True when the value is a map carrying a `stream` field.
pub(crate) fn is_streaming(resp: &Value) -> bool {
    let Value::Map(fields) = resp else {
        return false;
    };
    fields.borrow().contains_key("stream")
}

/// Write a streaming response, pulling events until the generator ends.
///
/// # Arguments
///
/// * `stream` — Socket to write to.
/// * `runtime` — Interpreter or VM, needed to call the generator per event.
/// * `resp` — The handler's response map.
///
/// # Returns
///
/// `Ok(())` once the generator ends, the bound is reached, or the peer disconnects. A
/// disconnect is not an error: a client closing an event stream is ordinary.
///
/// # Errors
///
/// Returns an error when the response shape is wrong or the generator itself fails.
pub(crate) fn write_streaming<W: Write>(
    stream: &mut W,
    runtime: &mut dyn Runtime,
    resp: &Value,
) -> Result<(), String> {
    let plan = super::http_stream_plan::Plan::from_response(resp)?;
    stream
        .write_all(plan.head().as_bytes())
        .map_err(|error| format!("write stream head: {error}"))?;
    // Flushed before the first event so a client sees the head immediately rather than
    // when the first event happens to arrive.
    stream
        .flush()
        .map_err(|error| format!("flush stream head: {error}"))?;
    super::http_stream_pump::pump(stream, runtime, &plan)
}
