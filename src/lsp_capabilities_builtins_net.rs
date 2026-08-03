//! Network builtins: HTTP client, HTTP/HTTPS servers, SMTP.
//!
//! Ported from `editor/vscode/lib/tool-data-network.js`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::builtins_net::TABLE;
//! assert!(TABLE.iter().any(|entry| entry.0 == "http_get"));
//! ```

use crate::lsp_capabilities::builtins::Entry;

/// Network builtins as `(name, params, summary)` rows.
#[rustfmt::skip]
pub const TABLE: &[Entry] = &[
    ("http_get", "url", "Run a blocking HTTP GET request."),
    ("http_head", "url", "Run a blocking HTTP HEAD request."),
    ("http_post", "url, body", "Run a blocking HTTP POST request."),
    ("http_request", "method, url[, body[, headers]]", "Run a blocking HTTP request."),
    ("http_serve", "port, handler", "Serve HTTP requests with a script handler."),
    ("http_serve_static", "port, root_dir", "Serve files beneath a directory."),
    ("https_serve", "port, certificate_pem, private_key_pem, handler", "Serve HTTPS with a PEM identity."),
    ("smtp_send", "host, port, from, to, subject, body", "Send an email over SMTP."),
];
