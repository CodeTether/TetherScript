//! Resolving a word to a declaration in the same file.
//!
//! Same-file resolution picks the *nearest visible* declaration, which is how
//! shadowing works: an inner `let x` in a nested block wins over an outer one,
//! because [`crate::lsp_capabilities::symbol::Symbol::visible_at`] has already
//! discarded declarations whose scope has closed. A top-level `fn` is hoisted, so
//! a call above its declaration still resolves — the layout used by most of the
//! repository's own examples, where `main` sits at the bottom.
//!
//! Cross-module resolution lives in
//! `src/lsp_capabilities_definition_module.rs`.
//!
//! # Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use tetherscript::lsp_capabilities::context::word_at;
//! use tetherscript::lsp_capabilities::definition_target::resolve;
//! use tetherscript::lsp_capabilities::docs::Docs;
//! use tetherscript::lsp_capabilities::jsonval::{obj, str_value};
//! use tetherscript::lsp_capabilities::request::Cursor;
//! use tetherscript::value::Value;
//!
//! let source = "fn twice(n) { n }\nlet v = twice(2)\n";
//! let mut store = HashMap::new();
//! store.insert("file:///a.tether".to_string(), source.to_string());
//! let params = obj(vec![
//!     ("textDocument", obj(vec![("uri", str_value("file:///a.tether"))])),
//!     ("position", obj(vec![("line", Value::Int(1)), ("character", Value::Int(9))])),
//! ]);
//! let docs = Docs::new(&store);
//! let cursor = Cursor::parse(&params, &docs).expect("resolves");
//! let word = word_at(cursor.text, cursor.offset).expect("word");
//! assert_eq!(resolve(&cursor, &docs, &word).unwrap().start, 3);
//! ```

use crate::lsp_capabilities::context::Word;
use crate::lsp_capabilities::definition::Target;
use crate::lsp_capabilities::definition_module::{alias_file, imported};
use crate::lsp_capabilities::docs::Docs;
use crate::lsp_capabilities::request::Cursor;
use crate::lsp_capabilities::symbol::{Symbol, SymbolKind};
use crate::lsp_capabilities::symbols;

/// Resolve the word under the cursor to a definition site.
///
/// # Arguments
///
/// * `cursor` — Resolved request cursor.
/// * `docs` — Open-document store.
/// * `word` — The word under the cursor and its `.` qualifier.
///
/// # Returns
///
/// `Some(Target)` for a local, parameter, same-file `fn`, import alias, or
/// imported member; `None` for a builtin, a keyword, or an unknown name, since
/// none of those has a source location the user can be sent to.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::symbols::collect;
/// // Builtins are not declared in the document, so they yield no target.
/// assert!(collect("println(1)").is_empty());
/// ```
pub fn resolve(cursor: &Cursor<'_>, docs: &Docs<'_>, word: &Word) -> Option<Target> {
    match word.qualifier.as_deref() {
        Some(qualifier) => imported(cursor, docs, qualifier, &word.text),
        None => same_file(cursor, &word.text).or_else(|| alias_file(cursor, docs, &word.text)),
    }
}

fn same_file(cursor: &Cursor<'_>, name: &str) -> Option<Target> {
    let symbol = nearest(cursor, name).filter(|s| s.kind != SymbolKind::Module)?;
    Some(Target {
        uri: cursor.uri.clone(),
        text: cursor.text.to_string(),
        start: symbol.offset,
        end: symbol.offset + symbol.name.len(),
    })
}

/// The visible declaration of `name` closest to the cursor.
///
/// # Arguments
///
/// * `cursor` — Resolved request cursor.
/// * `name` — Identifier to look up.
///
/// # Returns
///
/// `Some(symbol)` for the nearest in-scope declaration, `None` when the name is
/// not declared in this file or every declaration's scope has closed.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::symbols::collect;
/// assert_eq!(collect("let a = 1\nlet a = 2").len(), 2);
/// ```
pub fn nearest(cursor: &Cursor<'_>, name: &str) -> Option<Symbol> {
    symbols::collect(cursor.text)
        .into_iter()
        .filter(|symbol| symbol.name == name && symbol.visible_at(cursor.offset))
        .min_by_key(|symbol| cursor.offset.abs_diff(symbol.offset))
}
