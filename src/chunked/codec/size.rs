//! Strict hexadecimal chunk-size parsing.
//!
//! RFC 9112 §7.1 defines a chunk size as `1*HEXDIG` — bare hex digits, nothing else. This
//! module accepts exactly that and rejects every popular lenient extension, because each
//! one is a documented request-smuggling divergence: if a front end tolerates `+5` or
//! `0x5` or `5 ` and the back end does not (or reads a different number from it), the two
//! disagree about where the body ends and an attacker gets to inject a request.
//!
//! Rejected explicitly: a leading `+` or `-`, a `0x`/`0X` prefix, any leading or trailing
//! whitespace, an empty size, a non-hex byte, a value above [`MAX_CHUNK_BYTES`], and a
//! value that overflows `usize` (checked arithmetic, never a wrapping accumulate).
//!
//! Leading zeros *are* accepted (`007` is 7): RFC 9112 permits them and every
//! implementation agrees on their value, so they create no divergence.
//!
//! # Panics
//!
//! None. Parsing only iterates bytes and uses `checked_mul`/`checked_add`; there is no
//! indexing and no unchecked arithmetic.

use super::error::ChunkedError;
use super::limits::MAX_CHUNK_BYTES;

/// Parse the size field of a chunk-size line.
///
/// # Arguments
///
/// * `field` — Size bytes with any `;extension` already stripped.
///
/// # Returns
///
/// The declared chunk payload length in bytes.
///
/// # Errors
///
/// [`ChunkedError::Malformed`] for a sign, an `0x` prefix, whitespace, an empty or non-hex
/// field, a value over [`MAX_CHUNK_BYTES`], or an overflowing value. Never `Incomplete`:
/// the caller has already framed a complete line.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::{parse_chunk_size, ChunkedError};
///
/// assert_eq!(parse_chunk_size(b"1f").unwrap(), 31);
/// assert_eq!(parse_chunk_size(b"007").unwrap(), 7);
/// assert_eq!(parse_chunk_size(b"0").unwrap(), 0);
///
/// let rejected: [&[u8]; 8] = [b"+5", b"-5", b"0x5", b"5 ", b" 5", b"", b"5g", b"FFFFFFFF"];
/// for bad in rejected {
///     assert!(matches!(parse_chunk_size(bad), Err(ChunkedError::Malformed(_))), "{bad:?}");
/// }
/// ```
pub fn parse_chunk_size(field: &[u8]) -> Result<usize, ChunkedError> {
    reject_shape(field)?;
    let mut size: usize = 0;
    for byte in field {
        let digit = hex_digit(*byte)?;
        size = size
            .checked_mul(16)
            .and_then(|scaled| scaled.checked_add(digit))
            .ok_or_else(|| ChunkedError::malformed("chunk size overflows usize"))?;
        if size > MAX_CHUNK_BYTES {
            return Err(ChunkedError::malformed(format!(
                "chunk size exceeds {MAX_CHUNK_BYTES} bytes"
            )));
        }
    }
    Ok(size)
}

/// Reject size fields whose *shape* is illegal regardless of their value.
fn reject_shape(field: &[u8]) -> Result<(), ChunkedError> {
    if field.is_empty() {
        return Err(ChunkedError::malformed("chunk size is empty"));
    }
    if matches!(field.first().copied(), Some(b'+') | Some(b'-')) {
        return Err(ChunkedError::malformed("chunk size carries a sign"));
    }
    if field.len() > 1
        && field.first().copied() == Some(b'0')
        && matches!(field.get(1).copied(), Some(b'x') | Some(b'X'))
    {
        return Err(ChunkedError::malformed("chunk size has a 0x prefix"));
    }
    if field.iter().any(|byte| byte.is_ascii_whitespace()) {
        return Err(ChunkedError::malformed("chunk size contains whitespace"));
    }
    Ok(())
}

/// Value of one ASCII hex digit.
fn hex_digit(byte: u8) -> Result<usize, ChunkedError> {
    match byte {
        b'0'..=b'9' => Ok((byte - b'0') as usize),
        b'a'..=b'f' => Ok((byte - b'a') as usize + 10),
        b'A'..=b'F' => Ok((byte - b'A') as usize + 10),
        other => Err(ChunkedError::malformed(format!(
            "chunk size has non-hex byte {other:#04x}"
        ))),
    }
}
