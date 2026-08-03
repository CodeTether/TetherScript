//! Hover text for unqualified names: this file's symbols, builtins, keywords.
//!
//! What "is known about" a `let` binding is deliberately modest. TetherScript is
//! dynamically typed, so there is no declared type to report; the honest answer
//! is the declaration form (`let mut total`) plus the fact that its type is
//! whatever the initializer produced. Inventing a type here would be a guess, and
//! a hover that guesses is worse than one that admits the limit.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::hover_local::known_about;
//! use tetherscript::lsp_capabilities::symbol::SymbolKind;
//!
//! assert!(known_about(SymbolKind::Param).contains("Parameter"));
//! assert!(known_about(SymbolKind::Local).contains("dynamically typed"));
//! ```

use crate::lsp_capabilities::definition_target::nearest;
use crate::lsp_capabilities::docs::Docs;
use crate::lsp_capabilities::request::Cursor;
use crate::lsp_capabilities::symbol::SymbolKind;
use crate::lsp_capabilities::{builtins, hover_module, keywords};

/// Resolve an unqualified word.
///
/// # Arguments
///
/// * `cursor` — Resolved request cursor.
/// * `docs` — Open-document store, used when the word is an import alias.
/// * `name` — The hovered identifier.
///
/// # Returns
///
/// `Some((signature, description))` for an in-scope symbol, a builtin, a keyword,
/// or a constant; `None` for an unknown identifier.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::keywords::describe;
/// assert!(describe("move").is_some());
/// ```
pub fn unqualified(cursor: &Cursor<'_>, docs: &Docs<'_>, name: &str) -> Option<(String, String)> {
    if let Some(found) = local(cursor, docs, name) {
        return Some(found);
    }
    if let Some(entry) = builtins::lookup(name) {
        return Some((builtins::signature(entry), entry.2.to_string()));
    }
    keywords::describe(name).map(|text| (name.to_string(), text.to_string()))
}

fn local(cursor: &Cursor<'_>, docs: &Docs<'_>, name: &str) -> Option<(String, String)> {
    let symbol = nearest(cursor, name)?;
    if symbol.kind == SymbolKind::Module {
        return Some(hover_module::namespace(docs, cursor, &symbol));
    }
    Some((symbol.signature.clone(), known_about(symbol.kind)))
}

/// What the server can honestly say about a declaration kind.
///
/// # Arguments
///
/// * `kind` — Declaration kind.
///
/// # Returns
///
/// A one-line description, with no invented type information.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::hover_local::known_about;
/// use tetherscript::lsp_capabilities::symbol::SymbolKind;
/// assert!(known_about(SymbolKind::Function).contains("Function"));
/// ```
pub fn known_about(kind: SymbolKind) -> String {
    match kind {
        SymbolKind::Function => "Function declared in this file.",
        SymbolKind::Local => {
            "Binding declared in this file. TetherScript is dynamically typed, so \
its type is whatever the initializer evaluates to."
        }
        SymbolKind::Param => "Parameter of the enclosing function.",
        SymbolKind::Module => "Module namespace imported by this file.",
    }
    .to_string()
}
