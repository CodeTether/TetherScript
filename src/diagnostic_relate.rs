//! Related-span builders: the facility that lets one report name two places.
//!
//! Separate from [`crate::diagnostic::build`] because attaching *supporting*
//! locations is a distinct concern from constructing the report, and it is the
//! part ownership analysis depends on: "used after move" is unreadable without
//! "moved here".

use crate::diagnostic::label::Label;
use crate::diagnostic::render::Diagnostic;
use crate::diagnostic::span::Span;

impl Diagnostic {
    /// Sets the inline text printed after the primary caret run.
    ///
    /// # Arguments
    ///
    /// * `message` — short text such as `"value used here after move"`.
    ///
    /// # Returns
    ///
    /// `self`, for chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{Diagnostic, Span};
    /// let d = Diagnostic::error(Span::new(0, 1), "m").with_primary_label("here");
    /// assert_eq!(d.primary.message, "here");
    /// ```
    pub fn with_primary_label(mut self, message: impl Into<String>) -> Self {
        self.primary.message = message.into();
        self
    }

    /// Attaches a supporting location.
    ///
    /// # Arguments
    ///
    /// * `span` — where the supporting fact lives, e.g. the `move` expression.
    /// * `message` — what that location shows, e.g. `"value moved here"`.
    ///
    /// # Returns
    ///
    /// `self`, for chaining. Related labels render in insertion order, each as
    /// its own `note:` block with its own `-->` locator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{Diagnostic, Span};
    ///
    /// let d = Diagnostic::error(Span::new(20, 21), "use of moved value `x`")
    ///     .with_related(Span::new(8, 14), "value moved here");
    /// assert_eq!(d.related.len(), 1);
    /// assert_eq!(d.related[0].span, Span::new(8, 14));
    /// assert_eq!(d.related[0].message, "value moved here");
    /// ```
    pub fn with_related(mut self, span: Span, message: impl Into<String>) -> Self {
        self.related.push(Label::new(span, message));
        self
    }

    /// Attaches an already-built [`Label`] as a related location.
    ///
    /// # Arguments
    ///
    /// * `label` — the label to attach.
    ///
    /// # Returns
    ///
    /// `self`, for chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{Diagnostic, Label, Span};
    ///
    /// let d = Diagnostic::error(Span::at(3), "m")
    ///     .with_related_label(Label::new(Span::at(1), "first borrow here"));
    /// assert_eq!(d.related[0].message, "first borrow here");
    /// ```
    pub fn with_related_label(mut self, label: Label) -> Self {
        self.related.push(label);
        self
    }
}
