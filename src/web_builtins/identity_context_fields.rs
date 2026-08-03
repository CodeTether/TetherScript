//! Derivation of individual context fields from a request map.
//!
//! Split from [`super::identity_context`] so map assembly and field derivation are
//! separate concerns: the shape the script sees can change without touching the
//! precedence rules, and the precedence rules are testable on their own.

use std::collections::HashMap;
use std::rc::Rc;

use super::identity_headers::find;
use super::identity_request_id;
use super::identity_request_id_gen;
use crate::value::Value;

/// An optional header as a str, or `nil` when absent or blank.
///
/// # Arguments
///
/// * `headers` — Header map.
/// * `name` — Header name in any casing.
///
/// # Returns
///
/// `Value::Str` of the trimmed value, or `Value::Nil`. Blank collapses to `nil` so
/// a caller need only test for `nil`, never for `nil` *and* `""`.
pub(super) fn optional(headers: &HashMap<String, Value>, name: &str) -> Value {
    match find(headers, name) {
        Some(value) if !value.is_empty() => Value::Str(Rc::new(value)),
        _ => Value::Nil,
    }
}

/// Resolve the client address using the same precedence as `header_client_ip`.
///
/// # Arguments
///
/// * `req` — The request map, consulted for `remote_addr`.
/// * `headers` — Header map.
///
/// # Returns
///
/// Leftmost `X-Forwarded-For` entry, else `X-Real-IP`, else the request map's
/// `remote_addr`, else the empty string. Leftmost because each proxy appends, so
/// later entries are the proxies themselves.
///
/// See [`super::identity_headers`] for why that rule is restated rather than
/// called, and `header_client_ip` for why these headers are trustworthy only
/// behind a proxy that overwrites them.
pub(super) fn client_ip(req: &HashMap<String, Value>, headers: &HashMap<String, Value>) -> String {
    if let Some(forwarded) = find(headers, "x-forwarded-for") {
        let first = forwarded.split(',').next().unwrap_or("").trim();
        if !first.is_empty() {
            return first.to_string();
        }
    }
    if let Some(real) = find(headers, "x-real-ip").filter(|ip| !ip.is_empty()) {
        return real;
    }
    match req.get("remote_addr") {
        Some(Value::Str(addr)) => (**addr).clone(),
        _ => String::new(),
    }
}

/// The safe request id for these headers.
///
/// # Arguments
///
/// * `headers` — Header map, consulted for `X-Request-ID`.
///
/// # Returns
///
/// The incoming id when it passes [`identity_request_id::is_safe`], otherwise a
/// freshly generated one. Absent *and* rejected both take the generated path, so
/// an unsafe value is **replaced**, never sanitised: a truncated or escaped
/// attacker-chosen id is still attacker-influenced, and partial escaping is how
/// log-injection bugs recur.
pub(super) fn request_id(headers: &HashMap<String, Value>) -> String {
    match find(headers, "x-request-id") {
        Some(candidate) if identity_request_id::is_safe(&candidate) => candidate,
        _ => identity_request_id_gen::fresh(),
    }
}
