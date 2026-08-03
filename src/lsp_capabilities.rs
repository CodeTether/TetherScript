//! # Server-side LSP capabilities: completion, hover, go-to-definition
//!
//! Promotes the language intelligence that previously lived only in the VSCode
//! client (`editor/vscode/lib/*.js`) into the stdio server, so Neovim, Helix,
//! Zed, and anything else speaking LSP get the same features. Before this,
//! `src/lsp.rs` advertised only `textDocumentSync` and published diagnostics; see
//! AGENTS.md's Editor/LSP section, which says so plainly and warns against
//! describing the stdio server as feature-complete.
//!
//! ## What is implemented
//!
//! | Request | Handler | Reply |
//! |---------|---------|-------|
//! | `textDocument/completion` | [`completion::handle`] | `CompletionList` |
//! | `textDocument/hover` | [`hover::handle`] | `Hover` with markdown `MarkupContent`, or `null` |
//! | `textDocument/definition` | [`definition::handle`] | `Location`, or `null` |
//!
//! [`dispatch`] routes all three, so `src/lsp.rs` needs one match arm.
//!
//! ## Advertise only what you honour
//!
//! [`capabilities::entries`] returns exactly three providers. Advertising a
//! capability without a handler makes the editor open an empty popup, and users
//! read an empty popup as a broken language rather than a missing feature — a
//! worse outcome than the editor never offering the command at all. The list is
//! therefore derived from the handlers, not aspirational.
//!
//! ## Positions
//!
//! LSP positions are zero-based UTF-16 code units; the lexer's columns count
//! bytes. [`position`] converts once, at the request boundary, and everything
//! downstream works in byte offsets. This matters because an ASCII-only test
//! suite cannot detect getting it wrong.
//!
//! ## Failure policy
//!
//! Every handler is total. A closed document, an out-of-range position, a
//! document that does not lex, or an unreadable imported module all produce an
//! empty list or a null result — never a panic and never a JSON-RPC error. A
//! panicking language server takes the editor's language support down for the
//! rest of the session.
//!
//! ## Layout
//!
//! Submodules are declared by three grouping files — `group_core` (analysis),
//! `group_catalog` (documentation data), and `group_features` (handlers) — and
//! glob re-exported here so public paths stay flat.
//!
//! ## Quick start
//!
//! ```rust
//! use std::collections::HashMap;
//! use tetherscript::lsp_capabilities::dispatch;
//! use tetherscript::lsp_capabilities::jsonval::{obj, pointer, str_value, ValueText};
//! use tetherscript::value::Value;
//!
//! let mut docs = HashMap::new();
//! docs.insert("file:///a.tether".to_string(), "fn add(a, b) { a }\n".to_string());
//! let params = obj(vec![
//!     ("textDocument", obj(vec![("uri", str_value("file:///a.tether"))])),
//!     ("position", obj(vec![("line", Value::Int(0)), ("character", Value::Int(4))])),
//! ]);
//! let reply = dispatch("textDocument/hover", &params, &docs).expect("handled");
//! let text = pointer(&reply, &["contents", "value"]);
//! assert!(text.as_deref_str().unwrap().contains("add(a, b)"));
//! assert!(dispatch("textDocument/formatting", &params, &docs).is_none());
//! ```

use std::collections::HashMap;

use crate::value::Value;

#[path = "lsp_capabilities_group_catalog.rs"]
mod group_catalog;
#[path = "lsp_capabilities_group_core.rs"]
mod group_core;
#[path = "lsp_capabilities_group_features.rs"]
mod group_features;

pub use group_catalog::*;
pub use group_core::*;
pub use group_features::*;

/// Route one LSP request method to its handler.
///
/// The single call the stdio server needs: one arm in its `handle_request`
/// match, ahead of the existing `-32601 method not found` fallback.
///
/// # Arguments
///
/// * `method` — The JSON-RPC `method` string.
/// * `params` — The request's `params` object.
/// * `docs` — The server's URI → document-text map.
///
/// # Returns
///
/// `Some(result)` to send as the JSON-RPC `result` when `method` is one of
/// [`capabilities::METHODS`], or `None` when it is not, in which case the server
/// keeps its existing `method not found` behaviour. Returning `None` rather than
/// an error value leaves ownership of unknown methods with the server.
///
/// # Errors
///
/// Infallible. Handlers absorb every malformed request into an empty or null
/// result, so this never fails and never panics.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use tetherscript::lsp_capabilities::dispatch;
/// use tetherscript::value::Value;
///
/// let docs = HashMap::new();
/// assert!(dispatch("textDocument/completion", &Value::Nil, &docs).is_some());
/// assert!(dispatch("textDocument/rename", &Value::Nil, &docs).is_none());
/// ```
pub fn dispatch(method: &str, params: &Value, docs: &HashMap<String, String>) -> Option<Value> {
    match method {
        "textDocument/completion" => Some(completion::handle(params, docs)),
        "textDocument/hover" => Some(hover::handle(params, docs)),
        "textDocument/definition" => Some(definition::handle(params, docs)),
        _ => None,
    }
}
