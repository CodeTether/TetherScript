//! Chunk encoding: one chunk, and the terminating zero chunk.
//!
//! The wire form of a chunk is `<hex-size>CRLF<payload>CRLF`, and a body ends with a zero
//! chunk `0CRLF` followed by an optional trailer section and a final CRLF. Sizes are
//! emitted as bare lowercase hex with no sign, no `0x`, and no padding — the one shape
//! every intermediary reads identically.
//!
//! A zero-length payload is never encoded as a data chunk: `0\r\n` is the *terminator*, so
//! emitting it mid-stream would silently truncate the body for the peer. [`encode_chunk`]
//! therefore returns an empty vector for an empty slice, and the caller writes nothing.
//!
//! # Panics
//!
//! None. Encoding only appends to a `Vec`; there is no indexing and no arithmetic that can
//! overflow (`format!("{:x}")` handles any `usize`).

use super::error::ChunkedError;
use super::limits::MAX_CHUNK_BYTES;

/// Encode one data chunk.
///
/// # Arguments
///
/// * `payload` — Body bytes for this chunk.
///
/// # Returns
///
/// `<hex-len>\r\n<payload>\r\n`, or an empty vector when `payload` is empty, so that an
/// empty write can never be mistaken for the terminating zero chunk.
///
/// # Errors
///
/// [`ChunkedError::Malformed`] if `payload` is longer than [`MAX_CHUNK_BYTES`]; split it
/// across several chunks instead.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::encode_chunk;
///
/// assert_eq!(encode_chunk(b"hello").unwrap(), b"5\r\nhello\r\n".to_vec());
/// assert_eq!(encode_chunk(&[0u8; 255]).unwrap()[..4], b"ff\r\n"[..]);
/// assert!(encode_chunk(b"").unwrap().is_empty());
/// ```
pub fn encode_chunk(payload: &[u8]) -> Result<Vec<u8>, ChunkedError> {
    if payload.len() > MAX_CHUNK_BYTES {
        return Err(ChunkedError::malformed(format!(
            "chunk payload exceeds {MAX_CHUNK_BYTES} bytes"
        )));
    }
    if payload.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = format!("{:x}\r\n", payload.len()).into_bytes();
    out.extend_from_slice(payload);
    out.extend_from_slice(b"\r\n");
    Ok(out)
}
