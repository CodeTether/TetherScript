//! Encoding of the terminating zero chunk, with optional trailer fields.
//!
//! The terminator is `0CRLF`, then zero or more trailer field lines, then a final CRLF.
//! With no trailers that is exactly `0\r\n\r\n`.
//!
//! Trailer names and values are validated on the way *out*, not just on the way in. A value
//! containing CR or LF would end the trailer line early and let the caller inject arbitrary
//! extra fields, or a whole second message — response splitting, the mirror image of
//! request smuggling. Since a caller may build trailers from untrusted data, an injected
//! byte is rejected here rather than trusted.
//!
//! # Panics
//!
//! None. Only `Vec` appends and byte predicates; no indexing, no arithmetic.

use super::error::ChunkedError;
use super::limits::{MAX_TRAILERS, MAX_TRAILER_BYTES, MAX_TRAILER_LINE_BYTES};
use super::trailer_name::check_field;

/// Encode the zero chunk that ends a chunked body.
///
/// # Arguments
///
/// * `trailers` — Trailer fields as `(name, value)` pairs; empty for the common case.
///
/// # Returns
///
/// `0\r\n` followed by each `name: value\r\n`, followed by the final `\r\n`.
///
/// # Errors
///
/// [`ChunkedError::Malformed`] if there are more than [`MAX_TRAILERS`] fields, if any line
/// would exceed [`MAX_TRAILER_LINE_BYTES`], if the section would exceed
/// [`MAX_TRAILER_BYTES`], or if a name or value contains a control byte such as CR or LF.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::encode_last_chunk;
///
/// assert_eq!(encode_last_chunk(&[]).unwrap(), b"0\r\n\r\n".to_vec());
///
/// let with = encode_last_chunk(&[("X-Checksum".to_string(), "abc".to_string())]).unwrap();
/// assert_eq!(with, b"0\r\nX-Checksum: abc\r\n\r\n".to_vec());
///
/// // Injected CRLF is refused, not passed through.
/// let bad = [("X".to_string(), "a\r\nY: b".to_string())];
/// assert!(encode_last_chunk(&bad).is_err());
/// ```
pub fn encode_last_chunk(trailers: &[(String, String)]) -> Result<Vec<u8>, ChunkedError> {
    if trailers.len() > MAX_TRAILERS {
        return Err(ChunkedError::malformed(format!(
            "more than {MAX_TRAILERS} trailer fields"
        )));
    }
    let mut out = b"0\r\n".to_vec();
    let mut section = 0usize;
    for (name, value) in trailers {
        let line = check_field(name, value, MAX_TRAILER_LINE_BYTES)?;
        section += line.len();
        if section > MAX_TRAILER_BYTES {
            return Err(ChunkedError::malformed(format!(
                "trailer section exceeds {MAX_TRAILER_BYTES} bytes"
            )));
        }
        out.extend_from_slice(line.as_bytes());
    }
    out.extend_from_slice(b"\r\n");
    Ok(out)
}
