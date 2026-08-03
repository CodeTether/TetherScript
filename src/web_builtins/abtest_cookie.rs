//! Reading a named cookie out of a request's parsed cookie jar.
//!
//! The request map shape is the one `src/http_server_request_map.rs` builds:
//! `method`, `path`, `query`, `headers`, `body`. Cookies are read from the `cookies`
//! field when a caller has already parsed them, and otherwise from the raw `Cookie`
//! header via `abtest_cookie_header`, so a handler works whether or not it called
//! `cookie_parse` first.

use std::collections::HashMap;

use super::abtest_args as args;
use super::abtest_cookie_header as raw;
use crate::value::Value;

/// Read a named cookie from a request map.
///
/// # Arguments
///
/// * `request` — The request map a handler received.
/// * `name` — Cookie name to look for.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// `Ok(Some(value))` when the cookie is present with a non-empty value, and
/// `Ok(None)` when it is absent or empty. An empty value is treated as absent
/// because an expired cookie is commonly cleared by setting it to the empty string.
///
/// # Errors
///
/// Returns an error when `cookies` or `headers` is present but not a map.
pub(super) fn read(
    request: &HashMap<String, Value>,
    name: &str,
    label: &str,
) -> Result<Option<String>, String> {
    if let Some(value) = from_parsed(request, name, label)? {
        return Ok(Some(value));
    }
    Ok(raw::header(request, label)?.and_then(|header| raw::split(&header, name)))
}

/// Look in an already-parsed `cookies` map.
///
/// # Errors
///
/// Returns an error when `cookies` is present but not a map. A non-str value is not
/// an error, only a miss: a script may store richer cookie objects there, and this
/// group has no business dictating that shape.
fn from_parsed(
    request: &HashMap<String, Value>,
    name: &str,
    label: &str,
) -> Result<Option<String>, String> {
    let jar = match request.get("cookies") {
        None | Some(Value::Nil) => return Ok(None),
        Some(value) => args::map_arg(value, &format!("{label}: request `cookies`"))?,
    };
    match jar.get(name) {
        Some(Value::Str(text)) if !text.is_empty() => Ok(Some((**text).clone())),
        _ => Ok(None),
    }
}
