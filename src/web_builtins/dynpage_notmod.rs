//! The `304 Not Modified` response, and the decision to send one.
//!
//! # Shape
//!
//! The response map matches what `http_serve` already consumes: `status`,
//! `headers`, and `body`. `body` is the empty string rather than omitted — RFC 9110
//! forbids a body on 304, and omitting the key would make this map's shape differ
//! from every other response a handler builds.
//!
//! The cached validator is echoed back in `etag`, which RFC 9110 §15.4.5 requires
//! on a 304, so the client can keep revalidating the same representation.
//!
//! # nil versus a response
//!
//! A miss is `nil`, not an `Err`. Most conditional requests do not match, and that
//! is an ordinary outcome; an `Err` would make `?` abort a perfectly healthy
//! request path. `Err` is reserved for a malformed argument, which is a program
//! bug.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::dynpage_request::{find, headers_of};
use super::dynpage_validator::matches;
use crate::value::Value;

/// Decide whether a cached render is still fresh for this request.
///
/// # Arguments
///
/// * `cached_etag` — Validator stored alongside the cached render.
/// * `request` — The request map a handler received.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// A 304 response map when the client's `If-None-Match` matches, otherwise
/// [`Value::Nil`] so the caller renders or serves the cached body.
///
/// # Errors
///
/// Returns an error when `request` is not a map, or when its `headers` field is
/// present and not a map.
pub(super) fn decide(
    cached_etag: &str,
    request: &Value,
    label: &str,
) -> Result<Value, String> {
    let headers = headers_of(request, label)?;
    let Some(header) = find(&headers, "if-none-match") else {
        return Ok(Value::Nil);
    };
    if !matches(&header, cached_etag) {
        return Ok(Value::Nil);
    }
    Ok(response(cached_etag))
}

/// Build the 304 map, echoing the validator as RFC 9110 §15.4.5 requires.
fn response(etag: &str) -> Value {
    let mut headers = HashMap::new();
    headers.insert("etag".to_string(), Value::Str(Rc::new(etag.to_string())));
    let mut out = HashMap::new();
    out.insert("status".to_string(), Value::Int(304));
    out.insert(
        "headers".to_string(),
        Value::Map(Rc::new(RefCell::new(headers))),
    );
    out.insert("body".to_string(), Value::Str(Rc::new(String::new())));
    Value::Map(Rc::new(RefCell::new(out)))
}
