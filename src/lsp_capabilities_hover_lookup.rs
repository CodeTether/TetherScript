//! Deciding what a hovered word means.
//!
//! Order is most-specific first, and the order is the whole design:
//!
//! 1. `resource.name` → an owned-resource constructor.
//! 2. `alias.name` where `alias` is an import → that module's exported symbol.
//! 3. any other `owner.name` → a value or resource method.
//! 4. an in-scope symbol of this file — a `fn`, `let`, parameter, or import.
//! 5. a builtin function.
//! 6. a keyword or constant.
//!
//! A qualified name must never fall through to an unrelated global: `list.len()`
//! and `len(list)` are different things, and describing one as the other is a
//! documentation bug the user cannot see through. The user's own symbols outrank
//! builtins for the same reason — a local `fn map` shadows the builtin `map`, so
//! hover must describe what the code actually calls.
//!
//! Unqualified resolution lives in `src/lsp_capabilities_hover_local.rs`.
//!
//! # Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use tetherscript::lsp_capabilities::context::word_at;
//! use tetherscript::lsp_capabilities::docs::Docs;
//! use tetherscript::lsp_capabilities::hover_lookup::describe;
//! use tetherscript::lsp_capabilities::jsonval::{obj, str_value};
//! use tetherscript::lsp_capabilities::request::Cursor;
//! use tetherscript::value::Value;
//!
//! let mut store = HashMap::new();
//! store.insert("file:///a.tether".to_string(), "fn add(a, b) { a }".to_string());
//! let params = obj(vec![
//!     ("textDocument", obj(vec![("uri", str_value("file:///a.tether"))])),
//!     ("position", obj(vec![("line", Value::Int(0)), ("character", Value::Int(4))])),
//! ]);
//! let docs = Docs::new(&store);
//! let cursor = Cursor::parse(&params, &docs).expect("resolves");
//! let word = word_at(cursor.text, cursor.offset).expect("word");
//! assert_eq!(describe(&cursor, &docs, &word).unwrap().0, "add(a, b)");
//! ```

use crate::lsp_capabilities::context::Word;
use crate::lsp_capabilities::docs::Docs;
use crate::lsp_capabilities::hover_local::unqualified;
use crate::lsp_capabilities::request::Cursor;
use crate::lsp_capabilities::{hover_module, methods};

/// Resolve a hovered word to a `(signature, description)` pair.
///
/// # Arguments
///
/// * `cursor` — Resolved request cursor.
/// * `docs` — Open-document store, used to read imported modules.
/// * `word` — The hovered word and its qualifier.
///
/// # Returns
///
/// `Some((signature, description))` when the word is recognised, `None` for an
/// unknown identifier, which the handler turns into a null hover.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::builtins::lookup;
/// assert!(lookup("json_parse").is_some());
/// ```
pub fn describe(cursor: &Cursor<'_>, docs: &Docs<'_>, word: &Word) -> Option<(String, String)> {
    match word.qualifier.as_deref() {
        Some(qualifier) => qualified(cursor, docs, qualifier, &word.text),
        None => unqualified(cursor, docs, &word.text),
    }
}

fn qualified(
    cursor: &Cursor<'_>,
    docs: &Docs<'_>,
    qualifier: &str,
    name: &str,
) -> Option<(String, String)> {
    if qualifier == "resource" {
        if let Some(entry) = methods::factory(name) {
            return Some((entry.1.to_string(), entry.2.to_string()));
        }
    }
    if let Some(found) = hover_module::member(cursor, docs, qualifier, name) {
        return Some(found);
    }
    methods::lookup(name).map(|entry| (entry.1.to_string(), entry.2.to_string()))
}
