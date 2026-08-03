//! [`Severity`] — how serious a diagnostic is, plus its two renderings.
//!
//! One concern: the severity vocabulary shared by terminal output (`"error"`)
//! and LSP (`DiagnosticSeverity`, where 1 is Error and 2 is Warning).

/// Severity of a diagnostic.
///
/// # Variants
///
/// * `Error` — compilation cannot continue.
/// * `Warning` — suspicious but accepted.
/// * `Note` — supporting information.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::Severity;
///
/// let s = Severity::Warning;
/// match s {
///     Severity::Error => panic!("not this one"),
///     Severity::Warning => assert_eq!(s.as_str(), "warning"),
///     Severity::Note => panic!("not this one"),
/// }
/// assert_eq!(Severity::Error.lsp_code(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// A hard error.
    Error,
    /// A warning.
    Warning,
    /// Supporting information.
    Note,
}

impl Severity {
    /// The terminal prefix word.
    ///
    /// # Returns
    ///
    /// `"error"`, `"warning"`, or `"note"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::Severity;
    /// assert_eq!(Severity::Note.as_str(), "note");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }

    /// The LSP `DiagnosticSeverity` code.
    ///
    /// # Returns
    ///
    /// `1` for `Error`, `2` for `Warning`, `3` for `Note` (LSP "Information").
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::Severity;
    /// assert_eq!(Severity::Warning.lsp_code(), 2);
    /// assert_eq!(Severity::Note.lsp_code(), 3);
    /// ```
    pub fn lsp_code(&self) -> i64 {
        match self {
            Severity::Error => 1,
            Severity::Warning => 2,
            Severity::Note => 3,
        }
    }
}
