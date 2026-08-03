//! Constructors for [`Diagnostic`].
//!
//! Split from [`crate::diagnostic::render`] so that "how a report is built"
//! and "how a report is printed" are separate responsibilities. Every
//! constructor is total: none can fail, so none returns `Result`.

use crate::diagnostic::label::Label;
use crate::diagnostic::render::Diagnostic;
use crate::diagnostic::severity::Severity;
use crate::diagnostic::span::Span;

impl Diagnostic {
    /// Creates a diagnostic with an explicit severity and no inline label text.
    ///
    /// # Arguments
    ///
    /// * `severity` — how serious the problem is.
    /// * `span` — the region at fault.
    /// * `message` — the headline message. Name the offending thing here;
    ///   `"error"` alone is not an error message.
    ///
    /// # Returns
    ///
    /// A `Diagnostic` with no related labels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{Diagnostic, Severity, Span};
    /// let d = Diagnostic::new(Severity::Warning, Span::new(0, 2), "unused `x`");
    /// assert_eq!(d.severity, Severity::Warning);
    /// ```
    pub fn new(severity: Severity, span: Span, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            primary: Label::new(span, ""),
            related: Vec::new(),
        }
    }

    /// Shorthand for an error-severity diagnostic.
    ///
    /// # Arguments
    ///
    /// * `span` — the region at fault.
    /// * `message` — the headline message.
    ///
    /// # Returns
    ///
    /// A `Diagnostic` with [`Severity::Error`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{Diagnostic, Severity, Span};
    /// assert_eq!(
    ///     Diagnostic::error(Span::at(0), "m").severity,
    ///     Severity::Error
    /// );
    /// ```
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, span, message)
    }

    /// Shorthand for a warning-severity diagnostic.
    ///
    /// # Arguments
    ///
    /// * `span` — the region at fault.
    /// * `message` — the headline message.
    ///
    /// # Returns
    ///
    /// A `Diagnostic` with [`Severity::Warning`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{Diagnostic, Severity, Span};
    /// assert_eq!(
    ///     Diagnostic::warning(Span::at(0), "m").severity,
    ///     Severity::Warning
    /// );
    /// ```
    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, span, message)
    }
}
