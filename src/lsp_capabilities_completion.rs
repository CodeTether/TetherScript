//! `textDocument/completion` handler.
//!
//! Dispatches on [`crate::lsp_capabilities::completion_context::Context`] and
//! delegates item construction to the `completion_global` and
//! `completion_member` modules, so this file only owns request handling.
//!
//! The reply is always a `CompletionList` object (`isIncomplete: false`) rather
//! than a bare array. Both are legal, but the object form lets a future
//! incremental implementation flip `isIncomplete` without changing the wire
//! shape clients have already learned.
//!
//! # Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use tetherscript::lsp_capabilities::completion::handle;
//! use tetherscript::lsp_capabilities::jsonval::{field, obj, str_value};
//! use tetherscript::value::Value;
//!
//! let mut store = HashMap::new();
//! store.insert("file:///a.tether".to_string(), "let total = 1\n".to_string());
//! let params = obj(vec![
//!     ("textDocument", obj(vec![("uri", str_value("file:///a.tether"))])),
//!     ("position", obj(vec![("line", Value::Int(1)), ("character", Value::Int(0))])),
//! ]);
//! let reply = handle(&params, &store);
//! assert!(matches!(field(&reply, "items"), Value::List(_)));
//! ```

use std::collections::HashMap;

use crate::lsp_capabilities::completion_context::{classify, prefix, Context};
use crate::lsp_capabilities::docs::Docs;
use crate::lsp_capabilities::jsonval::{list, obj};
use crate::lsp_capabilities::request::Cursor;
use crate::lsp_capabilities::{completion_global, completion_member};
use crate::value::Value;

/// Answer a `textDocument/completion` request.
///
/// # Arguments
///
/// * `params` — The request's `params` object.
/// * `store` — The server's URI → document-text map.
///
/// # Returns
///
/// A `CompletionList` object. An unresolvable position yields an empty list
/// rather than an error: an editor shows "no suggestions" for an empty list, but
/// surfaces an error popup for a JSON-RPC error, which reads as a broken server.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use tetherscript::lsp_capabilities::completion::handle;
/// use tetherscript::lsp_capabilities::jsonval::field;
/// use tetherscript::value::Value;
///
/// let store = HashMap::new();
/// let reply = handle(&Value::Nil, &store);
/// match field(&reply, "items") {
///     Value::List(items) => assert!(items.borrow().is_empty()),
///     _ => panic!("items must be a list"),
/// }
/// ```
pub fn handle(params: &Value, store: &HashMap<String, String>) -> Value {
    let docs = Docs::new(store);
    let items = match Cursor::parse(params, &docs) {
        Some(cursor) => items_for(&cursor, &docs),
        None => Vec::new(),
    };
    obj(vec![
        ("isIncomplete", Value::Bool(false)),
        ("items", list(items)),
    ])
}

fn items_for(cursor: &Cursor<'_>, docs: &Docs<'_>) -> Vec<Value> {
    let typed = prefix(cursor.text, cursor.offset);
    match classify(cursor.text, cursor.offset) {
        Context::Factory => completion_member::factories(),
        Context::Member(owner) => completion_member::members(cursor, docs, &owner),
        Context::Global => completion_global::items(cursor.text, cursor.offset, typed),
    }
}
