//! Validation of an outgoing trailer field line.
//!
//! Split out from [`encode_last_chunk`] so that "is this field safe to serialise" is one
//! testable responsibility rather than a branch buried in the encoder.
//!
//! A field name must be a non-empty RFC 9110 token; a value must be free of CR, LF, and
//! NUL. Both restrictions exist for the same reason: any of those bytes lets caller data
//! terminate the line early and forge additional header fields, which is response
//! splitting.
//!
//! # Panics
//!
//! None. Only iteration over bytes and string formatting.
//!
//! [`encode_last_chunk`]: super::encode_last_chunk

use super::error::ChunkedError;

/// Validate one trailer field and render its wire line.
///
/// # Arguments
///
/// * `name` — Field name; must be a non-empty token.
/// * `value` — Field value; must contain no CR, LF, or NUL.
/// * `limit` — Maximum permitted length of the rendered line, CRLF included.
///
/// # Returns
///
/// `"{name}: {value}\r\n"`.
///
/// # Errors
///
/// [`ChunkedError::Malformed`] naming which of the name, the value, or the length was at
/// fault.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::check_trailer_field;
///
/// assert_eq!(check_trailer_field("X-A", "1", 64).unwrap(), "X-A: 1\r\n");
/// assert!(check_trailer_field("", "1", 64).is_err());
/// assert!(check_trailer_field("X A", "1", 64).is_err());
/// assert!(check_trailer_field("X-A", "1\n", 64).is_err());
/// assert!(check_trailer_field("X-A", "1", 4).is_err());
/// ```
pub fn check_field(name: &str, value: &str, limit: usize) -> Result<String, ChunkedError> {
    if name.is_empty() || !name.bytes().all(is_token_byte) {
        return Err(ChunkedError::malformed(format!(
            "trailer name {name:?} is not a token"
        )));
    }
    if value.bytes().any(|byte| matches!(byte, b'\r' | b'\n' | 0)) {
        return Err(ChunkedError::malformed(format!(
            "trailer {name} value contains a control byte"
        )));
    }
    let line = format!("{name}: {value}\r\n");
    if line.len() > limit {
        return Err(ChunkedError::malformed(format!(
            "trailer {name} exceeds {limit} bytes"
        )));
    }
    Ok(line)
}

/// Whether `byte` is an RFC 9110 `tchar`.
fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte)
}
