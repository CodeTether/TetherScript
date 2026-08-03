//! Per-field line encoding for `text/event-stream`.
//!
//! One function per field, each returning the field's complete line(s) including
//! the trailing `\n`. Nothing here appends the blank line that dispatches an
//! event — that belongs to [`super::event`], so no single function can be blamed
//! for both "what a field looks like" and "when an event ends".
//!
//! Exposed publicly because a caller with its own writer may want the line
//! encoders without [`super::EventStream`]'s buffer.

use super::error::SseError;

/// Encode a payload as one `data:` line per line of input.
///
/// # Arguments
///
/// * `payload` — Message body. May contain LF, CRLF, or lone CR.
///
/// # Returns
///
/// The `data:` lines. CRLF and lone CR are normalized to LF **first**, so a
/// carriage return can never reach the wire: a raw CR would end the line early
/// and the remainder would be reparsed as SSE fields. Splitting on LF is a
/// correctness requirement, not cosmetics — a raw newline inside one `data:` line
/// silently truncates the event at that point.
///
/// A payload of `"a\n"` deliberately yields a trailing empty `data:` line. That
/// round-trips: the client joins `data:` values with `\n`, then drops the final
/// `\n`, recovering `"a\n"` exactly. Dropping the empty line would lose the
/// author's trailing newline.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::fields::data_lines;
///
/// assert_eq!(data_lines("hello"), "data: hello\n");
/// assert_eq!(data_lines("a\nb"), "data: a\ndata: b\n");
/// assert_eq!(data_lines("a\r\nb"), "data: a\ndata: b\n");
/// assert_eq!(data_lines("\r"), "data: \ndata: \n");
/// ```
pub fn data_lines(payload: &str) -> String {
    let normalized = payload.replace("\r\n", "\n").replace('\r', "\n");
    normalized
        .split('\n')
        .map(|line| format!("data: {line}\n"))
        .collect()
}

/// Encode `retry: <ms>\n`.
///
/// # Arguments
///
/// * `ms` — Reconnection delay in **milliseconds**, which the spec requires to be
///   an integer. A fractional or unit-suffixed value is ignored wholesale by the
///   client, so the type is `u64` and no parsing is offered.
///
/// # Returns
///
/// The single `retry:` line. Infallible.
///
/// # Examples
///
/// ```rust
/// assert_eq!(tetherscript::sse::fields::retry_line(3000), "retry: 3000\n");
/// ```
pub fn retry_line(ms: u64) -> String {
    format!("retry: {ms}\n")
}

/// Encode a `:`-prefixed comment line, the standard keepalive.
///
/// # Arguments
///
/// * `text` — Comment body, one line.
///
/// # Returns
///
/// `": <text>\n"`. A comment is not an event: it carries no blank-line
/// terminator and dispatches nothing at the client.
///
/// # Errors
///
/// [`SseError::MultiLineField`] when `text` spans lines, since the second line
/// would be parsed as a real SSE field.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::fields::comment;
///
/// assert_eq!(comment("ping").unwrap(), ": ping\n");
/// assert!(comment("a\nb").is_err());
/// ```
pub fn comment(text: &str) -> Result<String, SseError> {
    if text.contains('\n') || text.contains('\r') {
        return Err(SseError::MultiLineField("comment"));
    }
    Ok(format!(": {text}\n"))
}
