//! Reading one chunk's payload and its trailing CRLF.
//!
//! The declared size is authoritative: exactly that many bytes are taken, then a CRLF is
//! required. Scanning for a CRLF instead of counting would let a payload that legitimately
//! contains `\r\n` — any text/event-stream frame, for instance — terminate its own chunk
//! early, which is precisely how a smuggled request gets in.
//!
//! Mismatches are diagnosed as early as the bytes allow: a wrong first terminator byte is
//! [`ChunkedError::Malformed`] as soon as it is visible, without waiting for the second,
//! since no continuation can repair it.
//!
//! # Panics
//!
//! None. Every read goes through `input.get(range)`, which returns `None` rather than
//! panicking for an out-of-range or inverted range, and offsets are built with
//! `checked_add` so a hostile size cannot wrap into a small in-range index.

use super::error::ChunkedError;

/// Take `size` payload bytes at `from`, then consume the required CRLF.
///
/// # Arguments
///
/// * `input` — Whole buffer being decoded.
/// * `from` — Offset of the first payload byte.
/// * `size` — Declared payload length, already bounds-checked by the size parser.
///
/// # Returns
///
/// The payload slice and the offset just past its CRLF.
///
/// # Errors
///
/// [`ChunkedError::Incomplete`] while the payload or its CRLF has not fully arrived.
/// [`ChunkedError::Malformed`] if the bytes after the payload are not CRLF, or if the offset
/// arithmetic would overflow.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::{chunk_payload, ChunkedError};
///
/// let (data, next) = chunk_payload(b"5\r\nhello\r\n0\r\n", 3, 5).unwrap();
/// assert_eq!(data, b"hello");
/// assert_eq!(next, 10);
///
/// assert!(matches!(chunk_payload(b"5\r\nhel", 3, 5), Err(ChunkedError::Incomplete)));
/// assert!(matches!(
///     chunk_payload(b"5\r\nhelloXX", 3, 5),
///     Err(ChunkedError::Malformed(_))
/// ));
/// ```
pub fn chunk_payload(
    input: &[u8],
    from: usize,
    size: usize,
) -> Result<(&[u8], usize), ChunkedError> {
    let end = from
        .checked_add(size)
        .ok_or_else(|| ChunkedError::malformed("chunk payload offset overflows usize"))?;
    let tail = input.get(end..).unwrap_or(&[]);
    if let Some(first) = tail.first().copied() {
        if first != b'\r' {
            return Err(ChunkedError::malformed("chunk payload lacks trailing CRLF"));
        }
        if matches!(tail.get(1).copied(), Some(second) if second != b'\n') {
            return Err(ChunkedError::malformed("chunk payload lacks trailing CRLF"));
        }
    }
    let payload = input
        .get(from..end)
        .filter(|_| tail.len() >= 2)
        .ok_or(ChunkedError::Incomplete)?;
    Ok((payload, end + 2))
}
