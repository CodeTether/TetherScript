//! `cors_preflight(policy, request)` — answer a preflight, or decline to.
//!
//! Returns `Ok(nil)` when the request is not a preflight, so a handler can call
//! this first, unconditionally, and fall through to its real work. Returning an
//! error only for a *refused* preflight keeps the two outcomes distinguishable:
//! `nil` means "not my business", an error means "this ask is not allowed".
//!
//! # Security
//!
//! A preflight from an origin that is not on the allow-list is refused rather
//! than answered with a header-less 204. A 204 with no `Allow-Origin` would be
//! rejected by the browser anyway, but refusing here gives the server an error to
//! log, which is the difference between noticing a probe and not.

use super::cors_preflight_check as check;
use super::cors_preflight_response as respond;
use super::cors_request::Request;
use super::{cors_origin, cors_policy_read, cors_request};
use crate::value::Value;

/// Explain a refused origin, naming it when one was sent.
fn refuse(request: &Request) -> String {
    match request.origin.as_deref() {
        Some(origin) => format!(
            "cors_preflight: origin `{origin}` is not allowed; origins are compared exactly, \
             so scheme, host, and port must all match an allow-list entry"
        ),
        None => "cors_preflight: preflight has no Origin header".to_string(),
    }
}

/// Decide the preflight answer.
///
/// # Arguments
///
/// * `policy` — A policy map from `cors_policy`.
/// * `request` — A request map as delivered to a handler.
///
/// # Returns
///
/// `Value::Nil` when `request` is not a preflight, else a 204 response map.
///
/// # Errors
///
/// Returns an error when either argument has the wrong shape, when the requesting
/// origin is absent or not allowed, or when the requested method or headers are
/// not on the allow-list. The message names the offending token.
pub(super) fn decide(policy: &Value, request: &Value) -> Result<Value, String> {
    let policy = cors_policy_read::read(policy, "cors_preflight")?;
    let request = cors_request::read(request, "cors_preflight")?;
    if !check::is_preflight(&request) {
        return Ok(Value::Nil);
    }
    let Some(origin) = cors_origin::allow(&policy, request.origin.as_deref()) else {
        return Err(refuse(&request));
    };
    let allow_headers = check::check(&policy, &request)?;
    Ok(respond::build(&policy, &origin, &allow_headers))
}

/// Whether a request is a preflight, for the `is_preflight` built-in.
///
/// # Arguments
///
/// * `request` — A request map as delivered to a handler.
///
/// # Returns
///
/// True only for `OPTIONS` carrying `Access-Control-Request-Method`.
///
/// # Errors
///
/// Returns an error when `request` is not a well-shaped request map.
pub(super) fn detect(request: &Value) -> Result<bool, String> {
    let request = cors_request::read(request, "is_preflight")?;
    Ok(check::is_preflight(&request))
}
