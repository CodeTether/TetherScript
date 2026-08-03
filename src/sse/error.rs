//! The single rejection type for SSE framing.
//!
//! Only two things are ever rejected, and both are injection risks rather than
//! formatting preferences: a single-line field that spans lines, and an `id` that
//! carries a control character. Everything else in this module is infallible, so
//! callers are not forced into `Result` where nothing can go wrong.

use std::fmt;

/// Why a frame could not be built.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::{Event, SseError};
///
/// // A newline in an id would let a caller forge event boundaries.
/// let err = Event::data("x").id("1\n2").render().unwrap_err();
/// assert_eq!(err, SseError::InvalidId);
/// assert!(err.to_string().contains("id"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseError {
    /// The `id` field contained CR, LF, or NUL.
    ///
    /// CR and LF would end the field and let the remainder be parsed as further
    /// SSE fields — including a blank line, which forges an event boundary. NUL
    /// is forbidden from the client's last-event-ID buffer by the spec. Rejecting
    /// is deliberate: silently stripping the bytes would hand back an id that no
    /// longer matches the caller's own records, so resume would replay the wrong
    /// position.
    InvalidId,
    /// A field that must be one line spanned several. Carries the field name,
    /// which is `"event"` for a named event or `"comment"` for a keepalive.
    MultiLineField(&'static str),
}

impl fmt::Display for SseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId => f.write_str(
                "sse: id must not contain CR, LF, or NUL; \
                 an injected newline would forge an event boundary",
            ),
            Self::MultiLineField(field) => write!(
                f,
                "sse: {field} must be a single line; a newline would forge a field boundary"
            ),
        }
    }
}

impl std::error::Error for SseError {}
