//! Whole-body encoding convenience.
//!
//! Most call sites stream chunk by chunk with [`encode_chunk`] and finish with
//! [`encode_last_chunk`]. This helper exists for the cases where the whole payload is
//! already in memory — a test, or a handler that wants chunked framing without a length —
//! and produces the same bytes those two calls would.
//!
//! A payload larger than [`MAX_CHUNK_BYTES`] is split across successive chunks rather than
//! rejected, since the caller's intent is unambiguous and the per-chunk bound exists to cap
//! a single allocation, not the message.
//!
//! # Panics
//!
//! None. `chunks` never yields an empty slice for a non-zero size, the size is a compile-
//! time non-zero constant, and only `Vec` appends follow.

use super::encode::encode_chunk;
use super::encode_last::encode_last_chunk;
use super::error::ChunkedError;
use super::limits::MAX_CHUNK_BYTES;

/// Encode `payload` as a complete chunked body, terminator included.
///
/// # Arguments
///
/// * `payload` — Entire body; an empty slice yields just the terminator.
/// * `trailers` — Trailer fields to place after the zero chunk.
///
/// # Returns
///
/// Data chunks followed by the zero chunk and trailer section.
///
/// # Errors
///
/// [`ChunkedError::Malformed`] if the trailers violate a bound or contain a control byte.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::{decode, encode_body};
///
/// let wire = encode_body(b"hello", &[]).unwrap();
/// assert_eq!(wire, b"5\r\nhello\r\n0\r\n\r\n".to_vec());
/// assert_eq!(decode(&wire).unwrap().payload, b"hello");
///
/// // An empty body is exactly the zero chunk.
/// assert_eq!(encode_body(b"", &[]).unwrap(), b"0\r\n\r\n".to_vec());
/// ```
pub fn encode_body(payload: &[u8], trailers: &[(String, String)]) -> Result<Vec<u8>, ChunkedError> {
    let mut out = Vec::new();
    for piece in payload.chunks(MAX_CHUNK_BYTES) {
        out.extend_from_slice(&encode_chunk(piece)?);
    }
    out.extend_from_slice(&encode_last_chunk(trailers)?);
    Ok(out)
}
