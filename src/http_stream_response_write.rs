//! Socket writes that treat a vanished peer as an outcome, not a failure.
//!
//! ## Why disconnect detection lives in its own module
//!
//! A closed browser tab is the common case for SSE, not the exception. The first
//! write after the peer goes away fails with `BrokenPipe`, `ConnectionReset`, or
//! `ConnectionAborted` (and on some platforms `EPIPE` surfaces only on the second
//! write, which is another reason every event is flushed rather than buffered —
//! buffering delays the discovery and lets the generator spin against a dead
//! socket). Classifying those kinds here, once, is what keeps one closed tab from
//! becoming a spinning server thread or an `unwrap` panic.

use std::io::{self, ErrorKind, Write};

/// Whether the socket is still usable after a write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Flow {
    /// The bytes were written and flushed; keep going.
    Open,
    /// The peer is gone. Stop the stream cleanly.
    Closed,
}

/// Write `bytes` and flush immediately.
///
/// # Arguments
///
/// * `out` — Destination socket.
/// * `bytes` — Payload, already framed by the caller.
///
/// # Returns
///
/// [`Flow::Open`] on success, [`Flow::Closed`] when the peer has gone away.
///
/// # Errors
///
/// Returns `Err` only for I/O failures that are *not* a disconnect — a full disk
/// on a redirected socket, for instance. Those are genuine faults and must not be
/// mistaken for a client closing its tab.
///
/// # Examples
///
/// ```text
/// if flush_all(&mut socket, b": keep-alive\n")? == Flow::Closed {
///     return Ok(Outcome::new(0, StopReason::Disconnected));
/// }
/// ```
pub(crate) fn flush_all<W: Write>(out: &mut W, bytes: &[u8]) -> Result<Flow, String> {
    if let Err(error) = out.write_all(bytes) {
        return classify(error);
    }
    match out.flush() {
        Ok(()) => Ok(Flow::Open),
        Err(error) => classify(error),
    }
}

/// Turn a write or flush failure into a flow decision.
///
/// # Arguments
///
/// * `error` — The failure reported by the socket.
///
/// # Returns
///
/// [`Flow::Closed`] for a peer-gone kind.
///
/// # Errors
///
/// Returns `Err` for every other kind, naming the underlying error.
fn classify(error: io::Error) -> Result<Flow, String> {
    if is_disconnect(&error) {
        return Ok(Flow::Closed);
    }
    Err(format!("http_serve: stream write failed: {error}"))
}

/// Report whether `error` means the peer closed the connection.
///
/// # Arguments
///
/// * `error` — The I/O error returned by a write or flush.
///
/// # Returns
///
/// `true` for the peer-gone kinds. Infallible.
pub(crate) fn is_disconnect(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::BrokenPipe
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected
            | ErrorKind::UnexpectedEof
    )
}
