//! Which completion set applies at the cursor.
//!
//! Ported from `editor/vscode/lib/completion-context.js`, with the regexes
//! replaced by direct byte inspection of the prefix before the cursor:
//!
//! - `resource.` → owned-resource constructors only.
//! - `alias.` where `alias` is an import → that module's exported names.
//! - any other `owner.` → value and resource methods.
//! - otherwise → keywords, constants, builtins, and in-scope symbols.
//!
//! Restricting member position to methods matters: offering `println` after a
//! `.` is worse than offering nothing, because it teaches the wrong syntax.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::completion_context::{Context, classify};
//!
//! assert!(matches!(classify("let x = ", 8), Context::Global));
//! assert!(matches!(classify("resource.", 9), Context::Factory));
//! assert!(matches!(classify("text.tr", 7), Context::Member(_)));
//! ```

use crate::lsp_capabilities::context::qualifier_before;

/// The completion set to serve at a cursor.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::completion_context::{Context, classify};
/// match classify("value.", 6) {
///     Context::Member(owner) => assert_eq!(owner, "value"),
///     Context::Factory => panic!("not a resource"),
///     Context::Global => panic!("after a dot"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Context {
    /// Cursor is not in member position: offer everything in scope.
    Global,
    /// Cursor follows `resource.`: offer owned-resource constructors.
    Factory,
    /// Cursor follows `owner.`: offer methods, or module exports when `owner`
    /// names an import alias.
    Member(String),
}

/// Classify the cursor position.
///
/// The word being typed is skipped before looking for a `.`, so both `math.`
/// and `math.ad` classify as `Member("math")`.
///
/// # Arguments
///
/// * `text` — Full document text.
/// * `offset` — Cursor byte offset.
///
/// # Returns
///
/// The applicable [`Context`].
///
/// # Errors
///
/// Infallible; an out-of-range offset is clamped and yields [`Context::Global`].
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::completion_context::{Context, classify};
/// assert_eq!(classify("resource.ti", 11), Context::Factory);
/// assert_eq!(classify("", 999), Context::Global);
/// ```
pub fn classify(text: &str, offset: usize) -> Context {
    let bytes = text.as_bytes();
    let offset = offset.min(bytes.len());
    let mut start = offset;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    match qualifier_before(text, start) {
        Some(owner) if owner == "resource" => Context::Factory,
        Some(owner) => Context::Member(owner),
        None => Context::Global,
    }
}

/// The partial word already typed at the cursor.
///
/// # Arguments
///
/// * `text` — Full document text.
/// * `offset` — Cursor byte offset.
///
/// # Returns
///
/// The identifier characters immediately before the cursor; empty when the
/// cursor follows whitespace or punctuation. Used as the filter prefix for
/// ranking, and echoed back as the completion item's `filterText`.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::completion_context::prefix;
/// assert_eq!(prefix("let tot", 7), "tot");
/// assert_eq!(prefix("let ", 4), "");
/// ```
pub fn prefix(text: &str, offset: usize) -> &str {
    let bytes = text.as_bytes();
    let offset = offset.min(bytes.len());
    let mut start = offset;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    &text[start..offset]
}
