//! Collect the names a document declares, with scopes.
//!
//! Symbol extraction walks the token stream rather than the AST for two
//! reasons. First, [`crate::ast`] keeps no source offsets, and go-to-definition
//! is meaningless without them. Second, an editor buffer is usually mid-edit:
//! the tokens of a document that fails to parse are still perfectly good, so
//! completion keeps working while the user is halfway through a line.
//!
//! Declaration forms live in sibling modules — `symbols_fn` for `fn` and its
//! parameters, `symbols_local` for `let`, `for`, and `import` — so each file
//! carries one responsibility.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::symbols::collect;
//!
//! let found = collect("fn add(a, b) { let sum = a }");
//! let names: Vec<&str> = found.iter().map(|s| s.name.as_str()).collect();
//! assert!(names.contains(&"add"));
//! assert!(names.contains(&"a"));
//! assert!(names.contains(&"sum"));
//! ```

use crate::lsp_capabilities::scan::{scan, Scanned};
use crate::lsp_capabilities::symbol::Symbol;
use crate::lsp_capabilities::{symbols_fn, symbols_local};
use crate::token::Token;

/// Collect every declaration in `text`.
///
/// # Arguments
///
/// * `text` — Full document text.
///
/// # Returns
///
/// All symbols in source order. An empty vector when the document does not lex.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::symbols::collect;
/// assert!(collect("\"unterminated").is_empty());
/// assert_eq!(collect("let x = 1").len(), 1);
/// ```
pub fn collect(text: &str) -> Vec<Symbol> {
    scan(text)
        .map(|scanned| from_scan(&scanned))
        .unwrap_or_default()
}

/// Collect declarations from an already-scanned document.
///
/// # Arguments
///
/// * `scanned` — Tokens plus byte offsets from [`scan`].
///
/// # Returns
///
/// All symbols in source order.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::scan::scan;
/// use tetherscript::lsp_capabilities::symbols::from_scan;
/// let scanned = scan("fn main() { }").expect("lexes");
/// assert_eq!(from_scan(&scanned).len(), 1);
/// ```
pub fn from_scan(scanned: &Scanned) -> Vec<Symbol> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    for index in 0..scanned.tokens.len() {
        match scanned.tokens[index].token {
            Token::LBrace => depth += 1,
            Token::RBrace => depth -= 1,
            Token::Fn => symbols_fn::declaration(scanned, index, depth, &mut out),
            Token::Let => symbols_local::binding(scanned, index, &mut out),
            Token::For => symbols_local::loop_binding(scanned, index, &mut out),
            Token::Import => symbols_local::import(scanned, index, &mut out),
            _ => {}
        }
    }
    out
}

/// Identifier text and byte offset at token `index`, if it is an identifier.
///
/// # Arguments
///
/// * `scanned` — Scanned document.
/// * `index` — Token index.
///
/// # Returns
///
/// `Some((name, byte_offset))`, or `None` for any other token or an index past
/// the end of the stream.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::scan::scan;
/// use tetherscript::lsp_capabilities::symbols::ident_at;
/// let scanned = scan("let value = 1").expect("lexes");
/// assert_eq!(ident_at(&scanned, 1), Some(("value", 4)));
/// assert_eq!(ident_at(&scanned, 0), None);
/// assert_eq!(ident_at(&scanned, 999), None);
/// ```
pub fn ident_at(scanned: &Scanned, index: usize) -> Option<(&str, usize)> {
    match scanned.tokens.get(index)?.token {
        Token::Ident(ref name) => Some((name.as_str(), scanned.offsets[index])),
        _ => None,
    }
}
