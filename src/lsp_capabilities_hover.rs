//! `textDocument/hover` handler.
//!
//! Returns LSP `MarkupContent` with `kind: "markdown"`, formatted the way the
//! VSCode client did in `hovers.js`: a fenced `tetherscript` code block holding
//! the signature, then a blank line, then the one-line description. Building the
//! markdown here rather than client-side is what lets Neovim, Helix, and Zed show
//! the identical popup.
//!
//! Lookup order is most-specific first, so a qualified name is never described
//! by an unrelated global of the same spelling. Resolution itself lives in
//! `src/lsp_capabilities_hover_lookup.rs`.
//!
//! # Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use tetherscript::lsp_capabilities::hover::handle;
//! use tetherscript::lsp_capabilities::jsonval::{obj, pointer, str_value, ValueText};
//! use tetherscript::value::Value;
//!
//! let mut store = HashMap::new();
//! store.insert("file:///a.tether".to_string(), "println(1)".to_string());
//! let params = obj(vec![
//!     ("textDocument", obj(vec![("uri", str_value("file:///a.tether"))])),
//!     ("position", obj(vec![("line", Value::Int(0)), ("character", Value::Int(2))])),
//! ]);
//! let reply = handle(&params, &store);
//! let value = pointer(&reply, &["contents", "value"]);
//! assert!(value.as_deref_str().unwrap().contains("println(...values)"));
//! ```

use std::collections::HashMap;

use crate::lsp_capabilities::context::word_at;
use crate::lsp_capabilities::docs::Docs;
use crate::lsp_capabilities::hover_lookup::describe;
use crate::lsp_capabilities::jsonval::{obj, str_value};
use crate::lsp_capabilities::position::offset_position;
use crate::lsp_capabilities::request::Cursor;
use crate::value::Value;

/// Answer a `textDocument/hover` request.
///
/// # Arguments
///
/// * `params` — The request's `params` object.
/// * `store` — The server's URI → document-text map.
///
/// # Returns
///
/// A `Hover` object with markdown contents and the hovered word's range, or
/// [`Value::Nil`] when there is nothing to describe. Null is the spec's "no
/// hover" answer, and it is also what keeps an unknown identifier from producing
/// an empty popup.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use tetherscript::lsp_capabilities::hover::handle;
/// use tetherscript::value::Value;
/// let store = HashMap::new();
/// assert!(matches!(handle(&Value::Nil, &store), Value::Nil));
/// ```
pub fn handle(params: &Value, store: &HashMap<String, String>) -> Value {
    let docs = Docs::new(store);
    let cursor = match Cursor::parse(params, &docs) {
        Some(cursor) => cursor,
        None => return Value::Nil,
    };
    let word = match word_at(cursor.text, cursor.offset) {
        Some(word) => word,
        None => return Value::Nil,
    };
    match describe(&cursor, &docs, &word) {
        Some((signature, description)) => {
            let range = span(cursor.text, word.start, word.end);
            obj(vec![
                ("contents", markup(&signature, &description)),
                ("range", range),
            ])
        }
        None => Value::Nil,
    }
}

fn markup(signature: &str, description: &str) -> Value {
    let body = format!("```tetherscript\n{signature}\n```\n\n{description}");
    obj(vec![
        ("kind", str_value("markdown")),
        ("value", str_value(&body)),
    ])
}

fn span(text: &str, start: usize, end: usize) -> Value {
    obj(vec![
        ("start", point(text, start)),
        ("end", point(text, end)),
    ])
}

fn point(text: &str, offset: usize) -> Value {
    let (line, character) = offset_position(text, offset);
    obj(vec![
        ("line", Value::Int(line as i64)),
        ("character", Value::Int(character as i64)),
    ])
}
