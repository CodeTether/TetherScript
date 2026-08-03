//! Core language builtins: values, conversion, and output.
//!
//! Ported verbatim from `editor/vscode/lib/tool-data-core.js` so the stdio
//! server and the VSCode client cannot drift apart.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::builtins_core::TABLE;
//! assert!(TABLE.iter().any(|entry| entry.0 == "println"));
//! ```

use crate::lsp_capabilities::builtins::Entry;

/// Core builtins as `(name, params, summary)` rows.
#[rustfmt::skip]
pub const TABLE: &[Entry] = &[
    ("Ok", "value", "Create a successful Result value."),
    ("Err", "message", "Create an error Result value."),
    ("assert", "condition[, message]", "Fail execution when a condition is false."),
    ("bytes", "value", "Convert a string or byte list to bytes."),
    ("eval", "source", "Evaluate source in the sandboxed tetherscript runtime."),
    ("global_defined", "name", "Return whether a global binding exists."),
    ("len", "value", "Return the length of a string, bytes, list, or map."),
    ("map", "", "Create an empty map."),
    ("parse_float", "text", "Parse a float and return a Result."),
    ("parse_int", "text", "Parse an integer and return a Result."),
    ("print", "...values", "Write values without a newline."),
    ("println", "...values", "Write values with a newline."),
    ("str", "value", "Convert a value to a string."),
    ("type_of", "value", "Return the runtime type name."),
];
