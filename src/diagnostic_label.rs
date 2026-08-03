//! [`Label`] — a span paired with the message that belongs to it.
//!
//! One concern: associating a byte [`Span`] with human text. Both the primary
//! label and every related label ("moved here") are the same shape, so there is
//! exactly one type for it.
//!
//! An empty message is meaningful: it means "underline this, but the headline
//! message already says everything", and rendering then omits the inline label.

use crate::diagnostic::span::Span;

/// A span plus its message.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{Label, Span};
///
/// let bare = Label::new(Span::new(0, 1), "");
/// assert_eq!(bare.text(), None);
///
/// let named = Label::new(Span::new(0, 1), "moved here");
/// assert_eq!(named.text(), Some("moved here"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    /// The region this label points at.
    pub span: Span,
    /// The message; empty means "no inline text".
    pub message: String,
}

impl Label {
    /// Creates a label.
    ///
    /// # Arguments
    ///
    /// * `span` — the region to point at.
    /// * `message` — inline text, possibly empty.
    ///
    /// # Returns
    ///
    /// The new `Label`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{Label, Span};
    /// let l = Label::new(Span::at(7), "expected `)`");
    /// assert_eq!(l.span, Span::at(7));
    /// assert_eq!(l.message, "expected `)`");
    /// ```
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }

    /// The inline text, or `None` when the message is empty.
    ///
    /// # Returns
    ///
    /// `Some(&str)` for a non-empty message, `None` otherwise, so renderers do
    /// not print a stray trailing space.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{Label, Span};
    /// assert_eq!(Label::new(Span::at(0), "hi").text(), Some("hi"));
    /// assert_eq!(Label::new(Span::at(0), "").text(), None);
    /// ```
    pub fn text(&self) -> Option<&str> {
        if self.message.is_empty() {
            None
        } else {
            Some(&self.message)
        }
    }
}
