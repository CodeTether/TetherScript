//! Construction of individual LSP `CompletionItem` objects.
//!
//! Mirrors `editor/vscode/lib/completion-items.js`, including its snippet
//! behaviour: callables insert `name($0)` with `insertTextFormat: 2` so the
//! caret lands between the parentheses. `sortText` carries the server's ranking
//! (see [`crate::lsp_capabilities::rank`]).
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::completion_item::{callable, plain};
//! use tetherscript::lsp_capabilities::jsonval::{field, ValueText};
//! use tetherscript::lsp_capabilities::rank::{Tier, sort_text};
//!
//! let item = callable("println", "println(...values)", "Write values with a newline.", 3, &sort_text(Tier::Builtin, 0));
//! assert_eq!(field(&item, "insertText").as_deref_str(), Some("println($0)"));
//! assert_eq!(field(&plain("let", 14, "50000"), "label").as_deref_str(), Some("let"));
//! ```

use crate::lsp_capabilities::jsonval::{obj, str_value};
use crate::value::Value;

/// A completion item that inserts a call with the caret inside the parentheses.
///
/// # Arguments
///
/// * `label` — Name shown and filtered on.
/// * `detail` — Signature shown beside the label.
/// * `documentation` — One-line description.
/// * `kind` — LSP `CompletionItemKind` number.
/// * `sort` — Precomputed `sortText`.
///
/// # Returns
///
/// A `CompletionItem` object with `insertTextFormat: 2` (snippet).
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::completion_item::callable;
/// use tetherscript::lsp_capabilities::jsonval::{field, ValueText};
/// let item = callable("map", "map()", "Create an empty map.", 3, "40000");
/// assert_eq!(field(&item, "insertText").as_deref_str(), Some("map($0)"));
/// ```
pub fn callable(label: &str, detail: &str, documentation: &str, kind: i64, sort: &str) -> Value {
    obj(vec![
        ("label", str_value(label)),
        ("kind", Value::Int(kind)),
        ("detail", str_value(detail)),
        ("documentation", str_value(documentation)),
        ("insertText", str_value(&format!("{label}($0)"))),
        ("insertTextFormat", Value::Int(2)),
        ("sortText", str_value(sort)),
    ])
}

/// A completion item that inserts its label verbatim.
///
/// Used for keywords, constants, and value bindings, none of which should gain
/// parentheses.
///
/// # Arguments
///
/// * `label` — Name inserted and shown.
/// * `kind` — LSP `CompletionItemKind` number.
/// * `sort` — Precomputed `sortText`.
///
/// # Returns
///
/// A `CompletionItem` object with `insertTextFormat: 1` (plain text).
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::completion_item::plain;
/// use tetherscript::lsp_capabilities::jsonval::{field, ValueText};
/// let item = plain("nil", 21, "60000");
/// assert_eq!(field(&item, "insertText").as_deref_str(), Some("nil"));
/// ```
pub fn plain(label: &str, kind: i64, sort: &str) -> Value {
    obj(vec![
        ("label", str_value(label)),
        ("kind", Value::Int(kind)),
        ("insertText", str_value(label)),
        ("insertTextFormat", Value::Int(1)),
        ("sortText", str_value(sort)),
    ])
}

/// A completion item with detail and documentation but no snippet.
///
/// # Arguments
///
/// * `label` — Name inserted and shown.
/// * `detail` — Signature or declaration text.
/// * `documentation` — One-line description.
/// * `kind` — LSP `CompletionItemKind` number.
/// * `sort` — Precomputed `sortText`.
///
/// # Returns
///
/// A `CompletionItem` object with `insertTextFormat: 1`.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::completion_item::described;
/// use tetherscript::lsp_capabilities::jsonval::{field, ValueText};
/// let item = described("total", "let total", "Local binding.", 6, "00000");
/// assert_eq!(field(&item, "detail").as_deref_str(), Some("let total"));
/// ```
pub fn described(label: &str, detail: &str, documentation: &str, kind: i64, sort: &str) -> Value {
    obj(vec![
        ("label", str_value(label)),
        ("kind", Value::Int(kind)),
        ("detail", str_value(detail)),
        ("documentation", str_value(documentation)),
        ("insertText", str_value(label)),
        ("insertTextFormat", Value::Int(1)),
        ("sortText", str_value(sort)),
    ])
}
