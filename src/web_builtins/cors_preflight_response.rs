//! The 204 response a successful preflight produces.
//!
//! 204 rather than 200: a preflight answer carries no body, and `204 No Content`
//! says so without a `Content-Length: 0` that some intermediaries mishandle. The
//! map shape — `status`, `headers`, `body` — is the one `http_serve` consumes.
//!
//! # Security
//!
//! `Access-Control-Allow-Headers` is rendered from the **policy's** list, never
//! from `Access-Control-Request-Headers`. See `cors_preflight_check`.

use std::collections::HashMap;

use super::cors_origin;
use super::cors_policy_read::Policy;
use super::cors_shape as shape;
use crate::value::Value;

/// Build the 204 preflight response.
///
/// # Arguments
///
/// * `policy` — The validated policy.
/// * `origin` — The value to echo, already decided by `cors_origin::allow`.
/// * `allow_headers` — Policy-approved request header names.
///
/// # Returns
///
/// A response map with status 204, `Access-Control-Allow-Origin`,
/// `-Allow-Methods`, `-Allow-Headers`, `Vary: Origin` when the answer varies by
/// origin, plus `-Allow-Credentials` and `-Max-Age` when configured.
pub(super) fn build(policy: &Policy, origin: &str, allow_headers: &[String]) -> Value {
    let mut headers: HashMap<String, Value> = HashMap::new();
    shape::put(&mut headers, "access-control-allow-origin", origin);
    shape::put(
        &mut headers,
        "access-control-allow-methods",
        &shape::join(&policy.methods),
    );
    shape::put(
        &mut headers,
        "access-control-allow-headers",
        &shape::join(allow_headers),
    );
    if cors_origin::varies(policy) {
        shape::put(&mut headers, "vary", "Origin");
    }
    if policy.credentials {
        shape::put(&mut headers, "access-control-allow-credentials", "true");
    }
    if let Some(seconds) = policy.max_age {
        shape::put(&mut headers, "access-control-max-age", &seconds.to_string());
    }
    shape::response(204, headers)
}
