//! The chunked decode driver: size line, payload, repeat, zero chunk, trailers.
//!
//! Incremental by construction. Every step either completes or returns
//! [`ChunkedError::Incomplete`], and `Incomplete` propagates out of this function without
//! any partial result, so a caller that appends more bytes and retries from the same offset
//! gets the same answer it would have got had the whole body arrived at once. Nothing is
//! consumed on `Incomplete`: [`DecodedBody::consumed`] only exists on the success path.
//!
//! Re-parsing from the start on each retry is deliberate. It costs a little work and buys
//! the guarantee that no partially-mutated state can be observed after a failure, which is
//! the state a smuggling attack tries to create.
//!
//! # Panics
//!
//! None. This function does no indexing of its own; framing, size parsing, payload slicing,
//! and trailer parsing each document their own freedom from panics, and the running total
//! uses `checked_add`.

use super::error::ChunkedError;
use super::limits::{MAX_BODY_BYTES, MAX_SIZE_LINE_BYTES};
use super::decoded::DecodedBody;
use super::extension::strip_extensions;
use super::line::crlf_line;
use super::payload::chunk_payload;
use super::size::parse_chunk_size;
use super::trailer::decode_trailers;

/// Decode a complete chunked body from the front of `input`.
///
/// # Arguments
///
/// * `input` — Buffer whose first byte is the first byte of a chunked body. Trailing bytes
///   beyond the body are ignored and reported via [`DecodedBody::consumed`].
///
/// # Returns
///
/// A [`DecodedBody`] with the concatenated payload, any trailers, and the byte count used.
///
/// # Errors
///
/// [`ChunkedError::Incomplete`] if `input` is a valid prefix of a chunked body; retry with
/// more bytes, having consumed none. [`ChunkedError::Malformed`] for a bad size, a missing
/// CRLF, a bad trailer, or any bound in this module's `limits` exceeded.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::{decode, ChunkedError};
///
/// let body = decode(b"4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n").unwrap();
/// assert_eq!(body.payload, b"Wikipedia");
/// assert!(body.trailers.is_empty());
/// assert_eq!(body.consumed, 24);
///
/// assert!(matches!(decode(b"4\r\nWi"), Err(ChunkedError::Incomplete)));
/// ```
pub fn decode(input: &[u8]) -> Result<DecodedBody, ChunkedError> {
    let mut at = 0usize;
    let mut payload: Vec<u8> = Vec::new();
    loop {
        let (line, after_size) = crlf_line(input, at, MAX_SIZE_LINE_BYTES, "chunk size line")?;
        let size = parse_chunk_size(strip_extensions(line))?;
        if size == 0 {
            let (trailers, consumed) = decode_trailers(input, after_size)?;
            return Ok(DecodedBody {
                payload,
                trailers,
                consumed,
            });
        }
        let (data, after_payload) = chunk_payload(input, after_size, size)?;
        if payload.len().checked_add(data.len()).unwrap_or(usize::MAX) > MAX_BODY_BYTES {
            return Err(ChunkedError::malformed(format!(
                "decoded body exceeds {MAX_BODY_BYTES} bytes"
            )));
        }
        payload.extend_from_slice(data);
        at = after_payload;
    }
}
