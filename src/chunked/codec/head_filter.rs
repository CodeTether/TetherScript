//! Which caller-supplied response headers a streaming head may carry.
//!
//! Split from the head builder so "is this header allowed through" is one testable rule.
//!
//! `content-length` is dropped, never forwarded. A message carrying both `Content-Length`
//! and `Transfer-Encoding: chunked` is the canonical request-smuggling vector: RFC 9112
//! §6.1 says `Transfer-Encoding` wins, but real intermediaries disagree, and a pair that
//! disagree let an attacker frame a second message inside the first. Rather than trust a
//! caller not to set it, the header is removed here.
//!
//! `transfer-encoding` is likewise dropped, because the head builder emits exactly one
//! authoritative `Transfer-Encoding: chunked`; a second copy is another framing ambiguity.
//! `connection` is dropped because a streaming response's connection handling belongs to the
//! server, not the handler.
//!
//! # Panics
//!
//! None. Only string comparison and byte predicates.

use super::error::ChunkedError;

/// Whether a caller header name is reserved by the streaming head writer.
///
/// # Arguments
///
/// * `name` — Header name; compared case-insensitively.
///
/// # Returns
///
/// `true` when the head writer owns this header and the caller's value must be discarded.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::is_reserved_header;
///
/// assert!(is_reserved_header("Content-Length"));
/// assert!(is_reserved_header("transfer-encoding"));
/// assert!(is_reserved_header("Connection"));
/// assert!(!is_reserved_header("Cache-Control"));
/// ```
pub fn is_reserved_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "content-length" | "transfer-encoding" | "connection"
    )
}

/// Reject a header whose name or value could forge extra fields.
///
/// # Arguments
///
/// * `name` — Header name; must be non-empty and free of CR, LF, NUL, and `:`.
/// * `value` — Header value; must be free of CR, LF, and NUL.
///
/// # Errors
///
/// [`ChunkedError::Malformed`] naming the offending half. Without this check a value
/// containing CRLF would end its line early and inject arbitrary headers — response
/// splitting.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::check_header;
///
/// assert!(check_header("Cache-Control", "no-store").is_ok());
/// assert!(check_header("X", "a\r\nContent-Length: 0").is_err());
/// assert!(check_header("", "v").is_err());
/// ```
pub fn check_header(name: &str, value: &str) -> Result<(), ChunkedError> {
    let bad = |byte: u8| matches!(byte, b'\r' | b'\n' | 0);
    if name.is_empty() || name.bytes().any(|byte| bad(byte) || byte == b':') {
        return Err(ChunkedError::malformed(format!(
            "response header name {name:?} is not usable"
        )));
    }
    if value.bytes().any(bad) {
        return Err(ChunkedError::malformed(format!(
            "response header {name} value contains a control byte"
        )));
    }
    Ok(())
}
