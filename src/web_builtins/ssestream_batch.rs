//! Batch folding: a list of event maps into one stream body.
//!
//! This is the form that works against today's `http_serve` with no server
//! change. The whole stream is produced up front and written as one
//! `Content-Length`-delimited body, so the client receives every event at once
//! and then sees end-of-stream.
//!
//! That is genuinely useful — replaying a finite history, or a short bounded
//! progress sequence — but it is not live streaming, and callers must not confuse
//! the two. A browser `EventSource` reading a batch response will consume every
//! event, observe the close, and reconnect after its retry delay, which for a
//! finite stream means it re-fetches the same events forever. Bounded batches are
//! therefore appropriate for `fetch`-based readers and for tests, while a live
//! feed needs the server change specified in `ssestream_spec`.

use crate::value::Value;

use super::ssestream_chunk as chunk;

/// Fold a list of event maps into one stream body.
///
/// # Arguments
///
/// * `events` — List of field maps, each framed by `ssestream_chunk::event`. An
///   empty list is valid and yields the empty string.
///
/// # Returns
///
/// The concatenated wire text of every event, in list order. Order is preserved
/// exactly, because SSE delivery order is the only ordering guarantee a client
/// has.
///
/// # Errors
///
/// Returns an error on the first invalid event, prefixed with that event's index
/// so a bad element in a long list is findable, rather than reporting only that
/// "an event" was malformed.
pub(super) fn body(events: &Value) -> Result<String, String> {
    let Value::List(items) = events else {
        return Err(format!(
            "sse_stream_response: events must be a list, got {}",
            events.type_name()
        ));
    };
    let mut out = String::new();
    for (index, item) in items.borrow().iter().enumerate() {
        let frame = chunk::event(item)
            .map_err(|error| format!("sse_stream_response: event {index}: {error}"))?;
        out.push_str(&frame);
    }
    Ok(out)
}
