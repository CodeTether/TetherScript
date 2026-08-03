//! Validation for the two single-line SSE fields, `event` and `id`.
//!
//! Split from [`super::fields`] because these are the only *fallible* encoders:
//! keeping rejection separate from formatting means the injection rules live in
//! exactly one place.

use super::error::SseError;

/// Encode `<field>: <text>\n`, rejecting embedded line breaks.
///
/// # Arguments
///
/// * `field` — Field name, used verbatim in the line and in the error.
/// * `text` — Field value, which must be a single line.
///
/// # Returns
///
/// The encoded line.
///
/// # Errors
///
/// [`SseError::MultiLineField`] when `text` contains CR or LF. Either byte ends
/// the field, so the remainder would be parsed as further SSE fields — and a
/// blank line among them would dispatch a forged event.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::validate::single_line;
///
/// assert_eq!(single_line("event", "tick").unwrap(), "event: tick\n");
/// assert!(single_line("event", "a\nb").is_err());
/// ```
pub fn single_line(field: &'static str, text: &str) -> Result<String, SseError> {
    if text.contains('\n') || text.contains('\r') {
        return Err(SseError::MultiLineField(field));
    }
    Ok(format!("{field}: {text}\n"))
}

/// Encode `id: <id>\n`, rejecting CR, LF, and NUL.
///
/// # Arguments
///
/// * `id` — Opaque resume token echoed back by the client in `Last-Event-ID`.
///
/// # Returns
///
/// The encoded `id:` line.
///
/// # Errors
///
/// [`SseError::InvalidId`] when `id` contains CR, LF, or NUL. The id is
/// **rejected rather than sanitised**: stripping bytes would hand the client a
/// token that no longer matches the caller's own records, so a later resume would
/// replay from the wrong position. NUL is separately forbidden from the client's
/// last-event-ID buffer by the spec.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::validate::id_line;
///
/// assert_eq!(id_line("42").unwrap(), "id: 42\n");
/// assert!(id_line("4\n2").is_err());
/// assert!(id_line("4\u{0}2").is_err());
/// ```
pub fn id_line(id: &str) -> Result<String, SseError> {
    if id.contains('\n') || id.contains('\r') || id.contains('\0') {
        return Err(SseError::InvalidId);
    }
    Ok(format!("id: {id}\n"))
}
