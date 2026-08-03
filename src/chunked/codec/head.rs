//! The response head for a chunked streaming response.
//!
//! # Why this never emits `Content-Length`
//!
//! A message MUST NOT carry both `Content-Length` and `Transfer-Encoding: chunked`. RFC
//! 9112 §6.1 declares such a message unrecoverable and says a proxy must reject or fix it,
//! because the two headers describe the body differently: one intermediary may frame the
//! body by the length, another by the chunk sizes, and the bytes one of them treats as body
//! the other treats as the *start of a new request*. That divergence is HTTP request
//! smuggling.
//!
//! There is also no honest length to send. A streaming response — Server-Sent Events, a log
//! tail — has no length known at head time; that is the whole reason for chunking.
//!
//! So this builder emits `Transfer-Encoding: chunked` and drops any caller
//! `Content-Length`, `Transfer-Encoding`, or `Connection` rather than trusting the caller
//! (see [`is_reserved_header`]). The invariant "never both" is structural, not a convention.
//!
//! # Panics
//!
//! None. Only `String` formatting and byte predicates; no indexing, no arithmetic.

use super::error::ChunkedError;
use super::head_filter::{check_header, is_reserved_header};

/// Build the head of a chunked streaming response.
///
/// The head is followed on the wire by chunks from [`encode_chunk`](super::encode_chunk)
/// and finally [`encode_last_chunk`](super::encode_last_chunk).
///
/// # Arguments
///
/// * `status` — HTTP status code for the status line.
/// * `reason` — Reason phrase; callers inside `http` pass `reason_phrase(status)`.
/// * `content_type` — Value for `Content-Type`, e.g. `text/event-stream`.
/// * `extra` — Additional headers. Reserved names are dropped, not honoured.
///
/// # Returns
///
/// The full head including its terminating blank line, ready to `write_all`.
///
/// # Errors
///
/// [`ChunkedError::Malformed`] if `content_type` or any `extra` name/value contains a
/// control byte that could forge further header fields.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::streaming_head;
///
/// let head = streaming_head(200, "OK", "text/event-stream", &[]).unwrap();
/// assert!(head.starts_with("HTTP/1.1 200 OK\r\n"));
/// assert!(head.contains("Transfer-Encoding: chunked\r\n"));
/// assert!(!head.to_ascii_lowercase().contains("content-length"));
/// assert!(head.ends_with("\r\n\r\n"));
///
/// // A caller-supplied Content-Length is discarded, never emitted alongside chunked.
/// let forged = [("Content-Length".to_string(), "0".to_string())];
/// let head = streaming_head(200, "OK", "text/plain", &forged).unwrap();
/// assert!(!head.to_ascii_lowercase().contains("content-length"));
/// ```
pub fn streaming_head(
    status: u16,
    reason: &str,
    content_type: &str,
    extra: &[(String, String)],
) -> Result<String, ChunkedError> {
    check_header("content-type", content_type)?;
    let mut head = format!("HTTP/1.1 {status} {reason}\r\n");
    head.push_str(&format!("Content-Type: {content_type}\r\n"));
    head.push_str("Transfer-Encoding: chunked\r\n");
    head.push_str("Connection: keep-alive\r\n");
    head.push_str("Cache-Control: no-cache\r\n");
    for (name, value) in extra {
        check_header(name, value)?;
        if !is_reserved_header(name) {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
    }
    head.push_str("\r\n");
    Ok(head)
}
