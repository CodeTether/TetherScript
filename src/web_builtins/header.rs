//! HTTP header built-ins for request handlers.
//!
//! Handlers receive a `headers` map from `http_serve` but no helpers, so today
//! every handler hand-rolls Bearer extraction, proxy address resolution, and
//! content negotiation. Each of those is easy to get subtly wrong, and two of
//! them are security-relevant.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `header_get(headers, name)` | `Result` of the value str, or `nil` when absent |
//! | `bearer_token(headers)` | `Result` of the token str |
//! | `client_ip(headers, remote_addr)` | address str |
//! | `accepts(headers, content_type)` | bool |
//! | `security_headers()` | map of recommended response headers |
//!
//! Every lookup is case-insensitive, because HTTP header names are
//! case-insensitive and an exact-case match silently misses a real header.
//!
//! # Examples
//!
//! ```tether
//! fn handle(req) {
//!     let token = bearer_token(req.headers)
//!     if token.is_err() { return unauthorized(token.err()) }
//!     if accepts(req.headers, "application/json") {
//!         return json(200, body, security_headers())
//!     }
//!     log_line(client_ip(req.headers, req.remote_addr))
//! }
//! ```
//!
//! # Security
//!
//! `client_ip` honors `X-Forwarded-For`, which is **client-controlled**. See
//! [`header_client_ip`] for why it must only be trusted behind a proxy that
//! overwrites it.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::{Env, Value};

#[path = "header_accept.rs"]
pub(super) mod header_accept;
#[path = "header_auth.rs"]
pub(super) mod header_auth;
#[path = "header_client_ip.rs"]
pub(super) mod header_client_ip;
#[path = "header_install.rs"]
pub(super) mod header_install;
#[path = "header_lookup.rs"]
pub(super) mod header_lookup;
#[path = "header_negotiate.rs"]
pub(super) mod header_negotiate;
#[path = "header_security.rs"]
pub(super) mod header_security;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    header_install::install(env);
}

/// Coerce a built-in argument to a string, naming the parameter on mismatch.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}
