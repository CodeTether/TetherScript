//! Response-head serialization for a streaming response.
//!
//! The head goes out **before** the generator runs even once, so a client sees
//! `200 OK` and its `EventSource` opens immediately. Deferring the head until the
//! first event would make an idle stream look like a hung server, which is the
//! same failure mode buffering causes.
//!
//! No `Content-Length` is ever written here — see [`super::chunk`] for why that
//! is a correctness requirement rather than an omission. Any `content-length` a
//! handler supplies is dropped for the same reason, and `connection` and
//! `transfer-encoding` are owned by the chosen [`super::chunk::Coding`] rather
//! than by the handler.

use std::io::Write;

use super::chunk::Coding;
use super::shape::StreamSpec;
use super::write::{flush_all, Flow};

/// Headers this module controls; a handler-supplied copy is ignored.
const RESERVED: [&str; 3] = ["content-length", "connection", "transfer-encoding"];

/// Write the status line and headers, then flush.
///
/// # Arguments
///
/// * `out` — Destination socket.
/// * `spec` — The parsed streaming response.
/// * `reason` — Reason phrase for `spec.status`.
///
/// # Returns
///
/// [`Flow::Open`] when the head reached the client, [`Flow::Closed`] when the
/// peer had already gone — a client that closes before reading the head is not an
/// error.
///
/// # Errors
///
/// Returns `Err` for a non-disconnect I/O failure.
///
/// # Examples
///
/// ```text
/// // 200, default headers, Coding::Close produces exactly:
/// // HTTP/1.1 200 OK\r\n
/// // Connection: close\r\n
/// // Cache-Control: no-cache\r\n            (order of the two below may vary)
/// // Content-Type: text/event-stream; charset=utf-8\r\n
/// // \r\n
/// ```
pub(crate) fn write_head<W: Write>(
    out: &mut W,
    spec: &StreamSpec,
    reason: &str,
) -> Result<Flow, String> {
    flush_all(out, render(spec, reason).as_bytes())
}

/// Render the head as a string, so tests can assert bytes without a socket.
///
/// # Arguments
///
/// * `spec` — The parsed streaming response.
/// * `reason` — Reason phrase for `spec.status`.
///
/// # Returns
///
/// The complete head including its terminating blank line. Infallible.
pub(crate) fn render(spec: &StreamSpec, reason: &str) -> String {
    let mut head = format!("HTTP/1.1 {} {reason}\r\n", spec.status);
    match spec.coding {
        Coding::Close => head.push_str("Connection: close\r\n"),
        Coding::Chunked => {
            head.push_str("Transfer-Encoding: chunked\r\nConnection: keep-alive\r\n")
        }
    }
    let mut names: Vec<&String> = spec.headers.keys().collect();
    names.sort();
    for name in names {
        if RESERVED.contains(&name.as_str()) {
            continue;
        }
        head.push_str(name);
        head.push_str(": ");
        head.push_str(&spec.headers[name]);
        head.push_str("\r\n");
    }
    head.push_str("\r\n");
    head
}
