//! `textDocument/definition` handler.
//!
//! Three jumps, in order of increasing difficulty:
//!
//! 1. a local binding or parameter in the same file;
//! 2. a `fn` declared in the same file — including one declared *below* the call
//!    site, since top-level `fn`s hoist;
//! 3. a symbol reached through an import alias — the case that requires resolving
//!    the module path and reading a second file, and the one users actually miss.
//!
//! Resolution lives in `src/lsp_capabilities_definition_target.rs`; this file
//! only shapes the reply.
//!
//! # Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use tetherscript::lsp_capabilities::definition::handle;
//! use tetherscript::lsp_capabilities::jsonval::{obj, pointer, str_value};
//! use tetherscript::value::Value;
//!
//! let mut store = HashMap::new();
//! store.insert("file:///a.tether".to_string(), "let total = 1\nprintln(total)\n".to_string());
//! let params = obj(vec![
//!     ("textDocument", obj(vec![("uri", str_value("file:///a.tether"))])),
//!     ("position", obj(vec![("line", Value::Int(1)), ("character", Value::Int(9))])),
//! ]);
//! let reply = handle(&params, &store);
//! assert!(matches!(pointer(&reply, &["range", "start", "line"]), Value::Int(0)));
//! ```

use std::collections::HashMap;

use crate::lsp_capabilities::context::word_at;
use crate::lsp_capabilities::definition_target::resolve;
use crate::lsp_capabilities::docs::Docs;
use crate::lsp_capabilities::jsonval::{obj, str_value};
use crate::lsp_capabilities::position::offset_position;
use crate::lsp_capabilities::request::Cursor;
use crate::value::Value;

/// A resolved definition site: which document, and which byte range in it.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::definition::Target;
/// let target = Target { uri: "file:///a.tether".into(), text: "let x = 1".into(), start: 4, end: 5 };
/// assert_eq!(&target.text[target.start..target.end], "x");
/// ```
pub struct Target {
    /// URI of the document containing the declaration.
    pub uri: String,
    /// Full text of that document, needed to convert offsets to positions.
    pub text: String,
    /// Byte offset of the first byte of the declared name.
    pub start: usize,
    /// Byte offset just past the declared name.
    pub end: usize,
}

/// Answer a `textDocument/definition` request.
///
/// # Arguments
///
/// * `params` — The request's `params` object.
/// * `store` — The server's URI → document-text map.
///
/// # Returns
///
/// A single `Location` object, or [`Value::Nil`] when nothing resolves. A single
/// `Location` rather than an array keeps the reply unambiguous: TetherScript has
/// no overloading, so a name has at most one declaration in scope.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use tetherscript::lsp_capabilities::definition::handle;
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
    match resolve(&cursor, &docs, &word) {
        Some(target) => location(&target),
        None => Value::Nil,
    }
}

fn location(target: &Target) -> Value {
    obj(vec![
        ("uri", str_value(&target.uri)),
        (
            "range",
            obj(vec![
                ("start", point(&target.text, target.start)),
                ("end", point(&target.text, target.end)),
            ]),
        ),
    ])
}

fn point(text: &str, offset: usize) -> Value {
    let (line, character) = offset_position(text, offset);
    obj(vec![
        ("line", Value::Int(line as i64)),
        ("character", Value::Int(character as i64)),
    ])
}
