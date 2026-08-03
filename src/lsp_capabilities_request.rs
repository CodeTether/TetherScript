//! Parsing `TextDocumentPositionParams` into a resolved cursor.
//!
//! All three handlers take the same params shape, so parsing lives here once.
//! The result carries the document text *and* the cursor's byte offset, because
//! everything downstream works in byte offsets and the UTF-16 conversion must
//! happen exactly once, at the boundary.
//!
//! # Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use tetherscript::lsp_capabilities::docs::Docs;
//! use tetherscript::lsp_capabilities::jsonval::{obj, str_value};
//! use tetherscript::lsp_capabilities::request::Cursor;
//! use tetherscript::value::Value;
//!
//! let mut store = HashMap::new();
//! store.insert("file:///a.tether".to_string(), "let x = 1".to_string());
//! let params = obj(vec![
//!     ("textDocument", obj(vec![("uri", str_value("file:///a.tether"))])),
//!     ("position", obj(vec![("line", Value::Int(0)), ("character", Value::Int(5))])),
//! ]);
//! let cursor = Cursor::parse(&params, &Docs::new(&store)).expect("resolves");
//! assert_eq!(cursor.offset, 5);
//! ```

use crate::lsp_capabilities::docs::Docs;
use crate::lsp_capabilities::jsonval::{pointer, ValueText};
use crate::lsp_capabilities::position::byte_offset;
use crate::value::Value;

/// A request's document, URI, and resolved cursor byte offset.
pub struct Cursor<'a> {
    /// Document URI, needed to build `Location` replies.
    pub uri: String,
    /// Full text of the open document.
    pub text: &'a str,
    /// Cursor position as a byte offset into `text`.
    pub offset: usize,
}

impl<'a> Cursor<'a> {
    /// Resolve `TextDocumentPositionParams` against the document store.
    ///
    /// # Arguments
    ///
    /// * `params` — The request's `params` object.
    /// * `docs` — Open-document store.
    ///
    /// # Returns
    ///
    /// `Some(Cursor)` when the URI names an open document and the line exists.
    /// `None` for a missing URI, a closed document, a non-integer position, or a
    /// line past the end of the document — every one of which is a client/server
    /// state mismatch that must produce a null result, never a panic, because a
    /// panicking language server takes the editor's language support down with
    /// it for the rest of the session.
    ///
    /// # Errors
    ///
    /// Infallible; all malformed input is reported as `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use tetherscript::lsp_capabilities::docs::Docs;
    /// use tetherscript::lsp_capabilities::request::Cursor;
    /// use tetherscript::value::Value;
    ///
    /// let store = HashMap::new();
    /// assert!(Cursor::parse(&Value::Nil, &Docs::new(&store)).is_none());
    /// ```
    pub fn parse(params: &Value, docs: &Docs<'a>) -> Option<Self> {
        let uri = pointer(params, &["textDocument", "uri"])
            .as_deref_str()?
            .to_string();
        let text = docs.text(&uri)?;
        let line = pointer(params, &["position", "line"]).as_index()?;
        let character = pointer(params, &["position", "character"]).as_index()?;
        let offset = byte_offset(text, line, character)?;
        Some(Self { uri, text, offset })
    }
}
