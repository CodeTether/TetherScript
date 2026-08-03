//! Hover text for module namespaces and their exported members.
//!
//! Ported from `editor/vscode/lib/module-hovers.js`. Two shapes:
//!
//! - hovering the alias itself reports the import declaration and how many names
//!   the module exports, which is the fastest way to notice a module you forgot
//!   to `export` from;
//! - hovering `alias.member` reports the member's own signature qualified by the
//!   alias, attributed to the importing path.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::module::exported_names;
//! assert_eq!(exported_names("export add\nfn add(a) { a }").len(), 1);
//! ```

use crate::lsp_capabilities::docs::Docs;
use crate::lsp_capabilities::request::Cursor;
use crate::lsp_capabilities::symbol::{Symbol, SymbolKind};
use crate::lsp_capabilities::{module, symbols, uri};

/// Hover text for an import alias.
///
/// # Arguments
///
/// * `docs` — Open-document store.
/// * `cursor` — Resolved request cursor, used to resolve the import path.
/// * `alias` — The [`SymbolKind::Module`] symbol under the cursor.
///
/// # Returns
///
/// The import declaration plus an export count, or an unresolved note when the
/// target file cannot be read. Reporting the failure is deliberate: silently
/// showing nothing looks identical to a server that does not support hover.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::symbol::{Symbol, SymbolKind};
/// let alias = Symbol::new("math", SymbolKind::Module, "import \"./m.tether\" as math", 0);
/// assert_eq!(alias.kind, SymbolKind::Module);
/// ```
pub fn namespace(docs: &Docs<'_>, cursor: &Cursor<'_>, alias: &Symbol) -> (String, String) {
    let description = match module_symbols(cursor, docs, &alias.detail) {
        Some(exports) => format!("{} explicit exports.", exports.len()),
        None => format!("Unresolved module `{}`.", alias.detail),
    };
    (alias.signature.clone(), description)
}

/// Hover text for `alias.member`.
///
/// # Arguments
///
/// * `cursor` — Resolved request cursor.
/// * `docs` — Open-document store.
/// * `qualifier` — Identifier before the `.`.
/// * `name` — Member name after the `.`.
///
/// # Returns
///
/// `Some((signature, description))` when `qualifier` is an import alias that
/// resolves and exports `name`; `None` otherwise, so hover falls through to the
/// method catalog.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::methods::lookup;
/// // Falling through to methods is what makes `text.trim()` still hover.
/// assert!(lookup("trim").is_some());
/// ```
pub fn member(
    cursor: &Cursor<'_>,
    docs: &Docs<'_>,
    qualifier: &str,
    name: &str,
) -> Option<(String, String)> {
    let alias = alias_named(cursor, qualifier)?;
    let exports = module_symbols(cursor, docs, &alias.detail)?;
    let found = exports.into_iter().find(|symbol| symbol.name == name)?;
    let signature = format!("{}.{}", alias.name, found.signature);
    Some((signature, format!("Exported by `{}`.", alias.detail)))
}

fn alias_named(cursor: &Cursor<'_>, qualifier: &str) -> Option<Symbol> {
    symbols::collect(cursor.text)
        .into_iter()
        .find(|symbol| symbol.kind == SymbolKind::Module && symbol.name == qualifier)
}

fn module_symbols(cursor: &Cursor<'_>, docs: &Docs<'_>, request: &str) -> Option<Vec<Symbol>> {
    let path = module::resolve(&uri::to_path(&cursor.uri)?, request)?;
    Some(module::exported_names(&docs.module_text(&path)?))
}
