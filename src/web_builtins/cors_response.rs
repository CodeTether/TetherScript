//! `cors_headers(policy, request)` — headers for an actual (non-preflight) response.
//!
//! Only the headers a real response needs are emitted: `Allow-Origin`,
//! `Allow-Credentials`, `Expose-Headers`, and `Vary`. `Allow-Methods`,
//! `Allow-Headers`, and `Max-Age` belong to the preflight answer only; sending
//! them on every response is wasted bytes that also invites the reader to believe
//! they do something here.
//!
//! # Security
//!
//! * A request with no `Origin`, or an origin not on the allow-list, yields an
//!   empty map — no `Allow-Origin` at all. The response is then simply not
//!   readable cross-origin, which is the correct outcome.
//! * `Vary: Origin` accompanies every echoed origin, so a shared cache cannot
//!   serve one origin's `Allow-Origin` to another.
//! * `Access-Control-Expose-Headers` is the *only* way script can read a response
//!   header beyond the CORS-safelisted set, so an entry here is a deliberate
//!   disclosure; it comes from the policy and is never reflected from the request.

use std::collections::HashMap;

use super::cors_shape as shape;
use super::{cors_origin, cors_policy_read, cors_request};
use crate::value::Value;

/// Build the response headers to merge into an actual response.
///
/// # Arguments
///
/// * `policy` — A policy map from `cors_policy`.
/// * `request` — A request map as delivered to a handler.
///
/// # Returns
///
/// A map of lowercase header names to values, empty when the origin is absent or
/// not allowed.
///
/// # Errors
///
/// Returns an error when either argument has the wrong shape.
pub(super) fn build(policy: &Value, request: &Value) -> Result<Value, String> {
    let policy = cors_policy_read::read(policy, "cors_headers")?;
    let request = cors_request::read(request, "cors_headers")?;
    let mut headers: HashMap<String, Value> = HashMap::new();
    let Some(origin) = cors_origin::allow(&policy, request.origin.as_deref()) else {
        // No header at all: not an empty value, and never a wildcard fallback.
        return Ok(shape::map(headers));
    };
    shape::put(&mut headers, "access-control-allow-origin", &origin);
    if cors_origin::varies(&policy) {
        shape::put(&mut headers, "vary", "Origin");
    }
    if policy.credentials {
        shape::put(&mut headers, "access-control-allow-credentials", "true");
    }
    if !policy.expose.is_empty() {
        shape::put(
            &mut headers,
            "access-control-expose-headers",
            &shape::join(&policy.expose),
        );
    }
    Ok(shape::map(headers))
}
