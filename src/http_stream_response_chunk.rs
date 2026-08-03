//! Body transfer coding for a streaming response.
//!
//! ## Why `Content-Length` must be absent
//!
//! `Content-Length` is a promise about a body length that is not yet known — the
//! generator has not run. Guessing is not an option: a value too small makes the
//! client stop reading mid-stream and treat the surplus as the start of another
//! response (request smuggling, in a proxy chain); a value too large makes the
//! client wait forever for bytes that never come, then report a truncated
//! response. HTTP/1.1 has exactly two honest framings for an unknown-length body,
//! and this module implements both.
//!
//! ## `Connection: close` versus chunked coding
//!
//! * [`Coding::Close`] (the default). The framing *is* the connection: end of
//!   body is end of socket. Every SSE client handles it, there is no per-event
//!   overhead, and it cannot be mis-framed. The cost is that the connection is
//!   not reusable, and a client cannot distinguish a completed stream from a
//!   crashed server — acceptable for SSE, whose clients reconnect by design and
//!   whose `retry:` field exists precisely to control that.
//! * [`Coding::Chunked`]. Keeps the connection reusable and marks a clean end of
//!   body with the terminating zero-length chunk, so a truncated stream is
//!   detectable. The cost is per-event framing bytes and the fact that every
//!   length must be hex — one decimal length and the client resynchronises on
//!   garbage. `tests/http_sse_stream.rs` reads the raw bytes to prove the hex
//!   framing and the terminator.
//!
//! Default is `Close` because it is the framing that cannot be got wrong, and
//! because `http_serve` closes after a stream anyway: the accept loop is
//! single-threaded, so there is no throughput to win by keeping the socket alive.

use std::collections::HashMap;

use crate::value::Value;

#[path = "http_stream_response_chunk_frame.rs"]
mod frame;

pub(crate) use frame::{finish, frame_bytes};

/// The end-of-body sequence: zero-length chunk plus empty trailer section.
///
/// Kept here rather than beside [`frame_bytes`] so the constant has one home and
/// no re-export exists purely for the tests to reach.
pub(crate) const TERMINATOR: &[u8] = b"0\r\n\r\n";

/// How the streamed body is delimited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Coding {
    /// End of body is end of connection. `Connection: close` in the head.
    Close,
    /// RFC 9112 chunked transfer coding.
    Chunked,
}

impl Coding {
    /// Read the optional `chunked` key.
    ///
    /// # Arguments
    ///
    /// * `map` — Borrowed streaming-response map.
    ///
    /// # Returns
    ///
    /// [`Coding::Chunked`] when `chunked` is `true`, otherwise [`Coding::Close`].
    ///
    /// # Errors
    ///
    /// Returns `Err` naming `chunked` when the value is present and not a bool.
    pub(crate) fn parse(map: &HashMap<String, Value>) -> Result<Self, String> {
        match map.get("chunked") {
            None | Some(Value::Nil) | Some(Value::Bool(false)) => Ok(Coding::Close),
            Some(Value::Bool(true)) => Ok(Coding::Chunked),
            Some(other) => Err(format!(
                "http_serve: stream response.chunked must be bool, got {}",
                other.type_name()
            )),
        }
    }
}
