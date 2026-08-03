//! Data-encoding builtins: JSON, Base64, hashing, URLs, templates.
//!
//! Ported from `editor/vscode/lib/tool-data-data.js`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::builtins_data::TABLE;
//! assert!(TABLE.iter().any(|entry| entry.0 == "json_parse"));
//! ```

use crate::lsp_capabilities::builtins::Entry;

/// Data builtins as `(name, params, summary)` rows.
#[rustfmt::skip]
pub const TABLE: &[Entry] = &[
    ("base64_decode", "text", "Decode Base64 text."),
    ("base64_encode", "text", "Encode text as Base64."),
    ("json_encode", "value", "Encode a value as compact JSON."),
    ("json_encode_pretty", "value", "Encode a value as formatted JSON."),
    ("json_parse", "text", "Parse JSON into tetherscript values."),
    ("sha256_hex", "text", "Compute a SHA-256 hex digest."),
    ("tera_render", "template, context[, escape]", "Render an optional Tera template with map data."),
    ("url_parse", "url", "Parse a URL into a map."),
];
