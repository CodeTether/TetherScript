//! The `capabilities` object to merge into the `initialize` result.
//!
//! **Advertise exactly what is honoured, and nothing more.** An editor takes the
//! `initialize` result literally: advertising `documentFormattingProvider` when
//! no formatter exists makes "Format Document" silently do nothing, and
//! advertising `signatureHelpProvider` without a handler makes a popup appear
//! and stay empty. Users read an empty popup as a *broken* language, not as a
//! missing feature — a strictly worse outcome than the editor never offering the
//! command at all. So this module advertises precisely three providers, matching
//! the three handlers in this crate:
//!
//! - `completionProvider` with `triggerCharacters: ["."]`, because member
//!   completion is only meaningful after a `.`;
//! - `hoverProvider`;
//! - `definitionProvider`.
//!
//! Deliberately **not** advertised, because they are not implemented here:
//! `documentSymbolProvider`, `documentLinkProvider`, `codeLensProvider`,
//! `signatureHelpProvider`, `referencesProvider`, `renameProvider`,
//! `documentFormattingProvider`, and `completionProvider.resolveProvider`
//! (there is no `completionItem/resolve` handler, and every item already ships
//! its own `detail` and `documentation`, so lazy resolution would buy nothing).
//!
//! `textDocumentSync` is *not* set here: it is owned by `src/lsp.rs`, which
//! already advertises full sync (`1`). The integrator merges these entries into
//! that same object.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::capabilities::entries;
//!
//! let names: Vec<&str> = entries().iter().map(|(name, _)| *name).collect();
//! assert_eq!(names, vec!["completionProvider", "hoverProvider", "definitionProvider"]);
//! ```

use crate::lsp_capabilities::jsonval::{list, obj, str_value};
use crate::value::Value;

/// The capability entries this crate honours, ready to merge into `capabilities`.
///
/// # Returns
///
/// `(key, value)` pairs to insert into the `initialize` result's `capabilities`
/// object, alongside the `textDocumentSync` entry `src/lsp.rs` already sets.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::capabilities::entries;
/// use tetherscript::value::Value;
/// let found = entries();
/// assert!(matches!(found[1].1, Value::Bool(true)));
/// ```
pub fn entries() -> Vec<(&'static str, Value)> {
    vec![
        ("completionProvider", completion()),
        ("hoverProvider", Value::Bool(true)),
        ("definitionProvider", Value::Bool(true)),
    ]
}

fn completion() -> Value {
    obj(vec![
        ("resolveProvider", Value::Bool(false)),
        ("triggerCharacters", list(vec![str_value(".")])),
    ])
}

/// The three request methods this crate answers.
///
/// Provided so the integrator's dispatch and this module cannot drift: if a
/// method is listed here it has a handler, and if it has a handler it is
/// advertised by [`entries`].
///
/// # Returns
///
/// The LSP method names, in the same order as [`entries`].
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::capabilities::METHODS;
/// assert!(METHODS.contains(&"textDocument/hover"));
/// assert_eq!(METHODS.len(), 3);
/// ```
pub const METHODS: &[&str] = &[
    "textDocument/completion",
    "textDocument/hover",
    "textDocument/definition",
];
