//! Chunked transfer-coding framing bytes (RFC 9112 §7.1).
//!
//! One chunk is `<hex-length>CRLF<data>CRLF`. Three details are easy to get
//! wrong and all three break the client silently:
//!
//! 1. The length is **hexadecimal**, without `0x` and without padding. A decimal
//!    length is read as hex by the client, so a 10-byte chunk announced as `10`
//!    makes it wait for 16 bytes.
//! 2. Each chunk carries its own trailing CRLF *after* the data, in addition to
//!    the CRLF after the length. Omitting it desynchronises every later chunk.
//! 3. The body ends with a zero-length chunk plus an empty trailer section:
//!    `0CRLFCRLF`. Without it the client sees a truncated body even though the
//!    server thinks it finished cleanly.
//!
//! A zero-length *data* chunk must never be emitted mid-stream, because `0` is
//! the terminator; [`frame_bytes`] therefore refuses empty payloads.

use super::super::write::Flow;
use super::TERMINATOR;

/// Frame `payload` as one chunk.
///
/// # Arguments
///
/// * `payload` — Event bytes. Must not be empty.
///
/// # Returns
///
/// `Some(bytes)` containing `<hex>CRLF<payload>CRLF`, or `None` when `payload`
/// is empty, since a zero-length chunk would be read as the terminator and end
/// the response early.
///
/// # Examples
///
/// ```text
/// frame_bytes(b"data: hi\n\n") == Some(b"a\r\ndata: hi\n\n\r\n".to_vec())
/// //                                   ^ 10 bytes, hex `a`, never `10`
/// frame_bytes(b"") == None
/// ```
pub(crate) fn frame_bytes(payload: &[u8]) -> Option<Vec<u8>> {
    if payload.is_empty() {
        return None;
    }
    let mut out = format!("{:x}\r\n", payload.len()).into_bytes();
    out.extend_from_slice(payload);
    out.extend_from_slice(b"\r\n");
    Some(out)
}

/// Write the terminating zero-length chunk, ignoring a peer that already left.
///
/// # Arguments
///
/// * `out` — Destination socket.
///
/// # Returns
///
/// `()`. A disconnect here is irrelevant — the body is already complete — so the
/// result of the write is deliberately discarded rather than reported.
///
/// # Errors
///
/// None. Errors are swallowed by design, as documented above.
pub(crate) fn finish<W: std::io::Write>(out: &mut W) {
    let _: Result<Flow, String> = super::super::write::flush_all(out, TERMINATOR);
}
