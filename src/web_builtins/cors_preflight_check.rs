//! Whether a request is a CORS preflight, and whether its ask is permitted.
//!
//! A preflight is an `OPTIONS` request carrying `Access-Control-Request-Method`
//! (Fetch, §CORS preflight request). Both conditions are required: an ordinary
//! `OPTIONS` — the kind a client uses to discover allowed methods — must not be
//! answered with a 204 that swallows the real handler, so the header is what
//! distinguishes them, not the method alone.
//!
//! # Security
//!
//! A requested method or header that is not on the allow-list is **refused**,
//! naming the offending token, rather than reflected back. Reflecting
//! `Access-Control-Request-Headers` into `Access-Control-Allow-Headers` — a
//! common shortcut — turns the allow-list into an echo chamber that permits
//! whatever the caller asks for, including headers the server never audited.

use super::cors_policy_read::Policy;
use super::cors_request::Request;
use super::cors_token;

/// Whether `request` is a preflight.
///
/// # Arguments
///
/// * `request` — The request under consideration.
///
/// # Returns
///
/// True only for `OPTIONS` carrying `Access-Control-Request-Method`.
pub(super) fn is_preflight(request: &Request) -> bool {
    request.method == "OPTIONS" && request.want_method.is_some()
}

/// Render an allow-list for an error message, naming the empty case.
fn listed(tokens: &[String]) -> String {
    if tokens.is_empty() {
        return "(none)".to_string();
    }
    tokens.join(", ")
}

/// Check the method and headers a preflight is asking for.
///
/// # Arguments
///
/// * `policy` — The validated policy.
/// * `request` — The preflight request.
///
/// # Returns
///
/// The allowed request-header names to echo, drawn from the *policy* rather than
/// from the request.
///
/// # Errors
///
/// Returns an error naming the method or header that is not on the allow-list,
/// along with what the policy does allow.
pub(super) fn check(policy: &Policy, request: &Request) -> Result<Vec<String>, String> {
    let wanted = cors_token::method(request.want_method.as_deref().unwrap_or_default());
    if !policy.methods.contains(&wanted) {
        return Err(format!(
            "cors_preflight: method `{wanted}` is not allowed; allowed methods are {}",
            listed(&policy.methods)
        ));
    }
    let asked = cors_token::header_list(request.want_headers.as_deref().unwrap_or_default());
    for name in asked {
        if !policy.headers.contains(&name) {
            return Err(format!(
                "cors_preflight: request header `{name}` is not allowed; \
                 allowed headers are {}",
                listed(&policy.headers)
            ));
        }
    }
    Ok(policy.headers.clone())
}
