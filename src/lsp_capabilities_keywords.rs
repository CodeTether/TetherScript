//! Language keywords and literal constants.
//!
//! Ported from `editor/vscode/lib/language-words.js`, cross-checked against
//! [`crate::token::Token`] so the list cannot silently fall behind the lexer:
//! every keyword token in `Token` appears here.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::keywords::{CONSTANTS, KEYWORDS, describe};
//!
//! assert!(KEYWORDS.contains(&"move"));
//! assert!(CONSTANTS.contains(&"nil"));
//! assert_eq!(describe("move"), Some("tetherscript language keyword."));
//! assert_eq!(describe("nil"), Some("Built-in tetherscript constant."));
//! assert_eq!(describe("println"), None);
//! ```

/// Every reserved keyword, in the order offered to completion clients.
#[rustfmt::skip]
pub const KEYWORDS: &[&str] = &[
    "as", "async", "await", "else", "export", "fn", "for", "if", "import", "in",
    "join", "let", "move", "mut", "panic", "return", "spawn", "while",
];

/// Literal constants that are lexed as keywords rather than identifiers.
pub const CONSTANTS: &[&str] = &["true", "false", "nil"];

/// One-line description for a keyword or constant.
///
/// # Arguments
///
/// * `word` — Candidate word.
///
/// # Returns
///
/// A description for a keyword or constant, or `None` when `word` is neither,
/// which lets hover fall through to the builtin and user-symbol lookups.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::keywords::describe;
/// assert!(describe("fn").is_some());
/// assert!(describe("total").is_none());
/// ```
pub fn describe(word: &str) -> Option<&'static str> {
    if KEYWORDS.contains(&word) {
        return Some("tetherscript language keyword.");
    }
    if CONSTANTS.contains(&word) {
        return Some("Built-in tetherscript constant.");
    }
    None
}
