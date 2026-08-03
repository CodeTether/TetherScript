//! Value method catalog for member completion and hover after a `.`.
//!
//! Ported from `editor/vscode/lib/method-data.js` and
//! `resource-method-*.js`. Rows are `(name, receiver_signature, summary)`;
//! the receiver is spelled out (`text.split(separator)`) because TetherScript is
//! dynamically typed, so the receiver's type is the only hint available at the
//! cursor.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::methods::{iter, lookup};
//!
//! assert_eq!(lookup("push").map(|entry| entry.1), Some("list.push(value)"));
//! assert!(iter().any(|entry| entry.0 == "unwrap_or"));
//! ```

use crate::lsp_capabilities::{methods_factory, methods_resource, methods_value};

/// One method row: name, receiver-qualified signature, one-line summary.
pub type Method = (&'static str, &'static str, &'static str);

/// Iterate every known value or resource method.
///
/// # Returns
///
/// An iterator over all method rows.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::methods::iter;
/// assert!(iter().any(|entry| entry.0 == "trim"));
/// ```
pub fn iter() -> impl Iterator<Item = &'static Method> {
    methods_value::TABLE
        .iter()
        .chain(methods_resource::CONTROLS)
        .chain(methods_resource::OPERATIONS)
}

/// Find one method by exact name.
///
/// # Arguments
///
/// * `name` — Method name as written after the `.`.
///
/// # Returns
///
/// `Some(row)` for a known method, `None` otherwise. Names are not unique per
/// receiver type in a dynamically typed language, so the first match wins and
/// value methods are searched before resource methods.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::methods::lookup;
/// assert!(lookup("keys").is_some());
/// assert!(lookup("not_a_method").is_none());
/// ```
pub fn lookup(name: &str) -> Option<&'static Method> {
    iter().find(|entry| entry.0 == name)
}

/// Find one `resource.*` factory by exact name.
///
/// # Arguments
///
/// * `name` — Factory name as written after `resource.`.
///
/// # Returns
///
/// `Some(row)` for a known factory, `None` otherwise.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::methods::factory;
/// assert!(factory("channel").is_some());
/// assert!(factory("push").is_none());
/// ```
pub fn factory(name: &str) -> Option<&'static Method> {
    methods_factory::FACTORIES
        .iter()
        .find(|entry| entry.0 == name)
}
