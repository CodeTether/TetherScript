//! Bounded CRLF line scanning over an in-memory chunked body.
//!
//! `src/http_server_reader.rs` reads request lines from a `BufRead` with an explicit byte
//! limit; this module is the same discipline applied to a slice, because a chunked body is
//! decoded from bytes already buffered rather than pulled a line at a time.
//!
//! Two deliberate strictnesses:
//!
//! * The terminator must be CRLF. A bare LF is [`ChunkedError::Malformed`], because a
//!   front end that accepts LF and a back end that requires CRLF disagree about where a
//!   chunk ends — the textbook request-smuggling divergence.
//! * The scan is bounded. Without a limit a peer that never sends a terminator forces an
//!   unbounded search and an unbounded buffer.
//!
//! # Panics
//!
//! None. `input.get(from..)` yields an empty slice rather than panicking for any `from`,
//! the window length is `min(rest.len(), limit + 2)` so slicing `rest` is always in range,
//! and the `at - 1` back-reference is guarded by the `at == 0` test just before it.

use super::error::ChunkedError;

/// Read one CRLF-terminated line starting at `from`.
///
/// # Arguments
///
/// * `input` — Whole buffer being decoded.
/// * `from` — Byte offset to start scanning at; may be past the end.
/// * `limit` — Maximum permitted line length, excluding the CRLF.
/// * `label` — Name of the construct, used in error messages.
///
/// # Returns
///
/// The line contents without its CRLF, and the offset of the first byte after the CRLF.
///
/// # Errors
///
/// [`ChunkedError::Incomplete`] if no CRLF is present yet but one could still arrive within
/// `limit`. [`ChunkedError::Malformed`] if the line exceeds `limit` or ends with a bare LF.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::{crlf_line, ChunkedError};
///
/// let (line, next) = crlf_line(b"5;a=b\r\nhello", 0, 256, "chunk size").unwrap();
/// assert_eq!(line, b"5;a=b");
/// assert_eq!(next, 7);
///
/// assert!(matches!(crlf_line(b"5", 0, 256, "chunk size"), Err(ChunkedError::Incomplete)));
/// assert!(matches!(
///     crlf_line(b"5\n", 0, 256, "chunk size"),
///     Err(ChunkedError::Malformed(_))
/// ));
/// ```
pub fn crlf_line<'a>(
    input: &'a [u8],
    from: usize,
    limit: usize,
    label: &str,
) -> Result<(&'a [u8], usize), ChunkedError> {
    let rest = input.get(from..).unwrap_or(&[]);
    let window = &rest[..rest.len().min(limit.saturating_add(2))];
    if let Some(at) = window.iter().position(|byte| *byte == b'\n') {
        if at == 0 || window.get(at - 1) != Some(&b'\r') {
            return Err(ChunkedError::malformed(format!(
                "{label} ends with bare LF"
            )));
        }
        return Ok((&window[..at - 1], from + at + 1));
    }
    // No LF yet. These bytes are still a valid prefix only if at most `limit` of them are
    // line content, plus at most the CR of a terminator that has not fully arrived.
    let content = window
        .len()
        .saturating_sub(usize::from(window.last() == Some(&b'\r')));
    if content > limit {
        return Err(ChunkedError::malformed(format!(
            "{label} exceeds {limit} bytes"
        )));
    }
    Err(ChunkedError::Incomplete)
}
