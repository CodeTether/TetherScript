//! Pulling events from a script generator and writing each as it arrives.
//!
//! The flush after every event is the entire point: without it the runtime's buffer holds
//! the first event until the socket closes, and a client sees nothing until the stream ends
//! — which is exactly the behaviour SSE exists to avoid.

use std::io::Write;

use super::http_stream_plan::Plan;
use crate::value::{Runtime, Value};

/// Pull events until the generator ends, the bound is reached, or the peer disconnects.
///
/// # Errors
///
/// Returns an error when the generator fails or yields a value that is not text. A write
/// failure is *not* an error: it means the client hung up, which is ordinary for a stream.
pub(crate) fn pump<W: Write>(
    stream: &mut W,
    runtime: &mut dyn Runtime,
    plan: &Plan,
) -> Result<(), String> {
    let mut written = 0i64;
    while written < plan.max_events() {
        let event = runtime.invoke(plan.generator(), &[])?;
        // Nil ends the stream; that is the generator's only way to say "finished".
        if matches!(event, Value::Nil) {
            break;
        }
        let text = text_of(&event)?;
        if !write_event(stream, &text, plan.chunked()) {
            // The peer closed. Stop quietly rather than reporting an error a handler
            // could not have prevented.
            return Ok(());
        }
        written += 1;
    }
    finish(stream, plan.chunked());
    Ok(())
}

/// Write one event, returning false when the peer has gone.
fn write_event<W: Write>(stream: &mut W, text: &str, chunked: bool) -> bool {
    let bytes = if chunked {
        // Hex length, per RFC 9112. A decimal length is read as hex by the client, which
        // then waits for a payload that never arrives.
        format!("{:x}\r\n{text}\r\n", text.len()).into_bytes()
    } else {
        text.as_bytes().to_vec()
    };
    stream.write_all(&bytes).is_ok() && stream.flush().is_ok()
}

/// Write the terminating bytes, ignoring a peer that has already left.
fn finish<W: Write>(stream: &mut W, chunked: bool) {
    if chunked {
        // Without this a client reports a truncated body rather than a clean end.
        let _ = stream.write_all(b"0\r\n\r\n");
    }
    let _ = stream.flush();
}

/// Coerce an event to text.
///
/// # Errors
///
/// Returns an error naming the type, since a stream of maps or lists has no defined framing
/// and silently rendering a debug form would ship malformed events.
fn text_of(event: &Value) -> Result<String, String> {
    match event {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!(
            "http_serve: a stream generator must yield str or nil, got {}",
            other.type_name()
        )),
    }
}
