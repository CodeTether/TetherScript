//! Catalog of built-in functions, ported from `editor/vscode/lib/tool-data-*.js`.
//!
//! Each entry is a `(name, params, summary)` triple. Splitting the parameter
//! list out of the signature avoids repeating the function name in every row,
//! which keeps rows on one line and each catalog file inside the repository's
//! 50-line limit.
//!
//! The catalog is deliberately data, not code: it is the same table the VSCode
//! client shipped in JavaScript, so moving it server-side gives Neovim, Helix,
//! and Zed identical completion detail and hover text without duplicating it in
//! two languages.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::builtins::{lookup, signature};
//!
//! let entry = lookup("println").expect("println is a builtin");
//! assert_eq!(signature(entry), "println(...values)");
//! assert!(lookup("no_such_builtin").is_none());
//! ```

use crate::lsp_capabilities::{
    builtins_browser, builtins_core, builtins_data, builtins_files, builtins_net, builtins_system,
    builtins_terminal,
};

/// One catalog row: function name, parameter list, one-line summary.
pub type Entry = (&'static str, &'static str, &'static str);

/// Every builtin catalog, grouped by concern.
///
/// # Returns
///
/// A slice of per-category tables.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::builtins::tables;
/// assert!(tables().len() >= 7);
/// ```
pub fn tables() -> &'static [&'static [Entry]] {
    &[
        builtins_core::TABLE,
        builtins_data::TABLE,
        builtins_files::TABLE,
        builtins_net::TABLE,
        builtins_system::TABLE,
        builtins_terminal::TABLE,
        builtins_browser::TABLE,
    ]
}

/// Iterate every builtin in the catalog.
///
/// # Returns
///
/// An iterator over all rows across all categories.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::builtins::iter;
/// assert!(iter().any(|entry| entry.0 == "fs_read"));
/// ```
pub fn iter() -> impl Iterator<Item = &'static Entry> {
    tables().iter().flat_map(|table| table.iter())
}

/// Find one builtin by exact name.
///
/// # Arguments
///
/// * `name` — Builtin function name, e.g. `"json_parse"`.
///
/// # Returns
///
/// `Some(entry)` when the name is a builtin, `None` otherwise.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::builtins::lookup;
/// assert_eq!(lookup("len").map(|entry| entry.1), Some("value"));
/// ```
pub fn lookup(name: &str) -> Option<&'static Entry> {
    iter().find(|entry| entry.0 == name)
}

/// Render a catalog row as a call signature.
///
/// # Arguments
///
/// * `entry` — Catalog row.
///
/// # Returns
///
/// `"name(params)"`.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::builtins::signature;
/// assert_eq!(signature(&("map", "", "Create an empty map.")), "map()");
/// ```
pub fn signature(entry: &Entry) -> String {
    format!("{}({})", entry.0, entry.1)
}
