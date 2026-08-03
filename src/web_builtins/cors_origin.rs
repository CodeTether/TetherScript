//! The allow-origin decision, and why `Vary: Origin` travels with it.
//!
//! # Security
//!
//! * **Exact comparison only.** An origin matches when its scheme, host, and port
//!   are byte-for-byte equal to a list entry. Suffix and substring matching are
//!   never used: `https://evil-example.com` *ends with* `example.com`, and
//!   `https://example.com.evil.net` *contains* it, so either test hands the
//!   allow-list to anyone who can register a domain name.
//! * **No match means no header.** A disallowed origin gets no `Allow-Origin`
//!   header at all — not an empty value, which some clients mis-handle, and
//!   certainly not a wildcard fallback, which would defeat the list entirely.
//! * **`Vary: Origin` whenever an origin is echoed.** The response then depends
//!   on a request header, so a shared cache keyed only on the URL would serve
//!   origin A's response — carrying `Allow-Origin: A` — to origin B, or cache B's
//!   rejection and replay it to A.
//! * **Credentials never pair with `*`.** `cors_config` rejects that combination
//!   at construction, so by the time this runs the wildcard branch is known to be
//!   credential-free.

use super::cors_policy_read::Policy;

/// The value to echo in `Access-Control-Allow-Origin`.
///
/// # Arguments
///
/// * `policy` — The validated policy.
/// * `origin` — The request's `Origin` header, when it sent one.
///
/// # Returns
///
/// `Some(value)` when the origin is allowed, else `None` so the caller emits no
/// header. For a wildcard policy the value is `*`; otherwise it is the requesting
/// origin echoed exactly, which is required when credentials are enabled and is
/// harmless when they are not.
pub(super) fn allow(policy: &Policy, origin: Option<&str>) -> Option<String> {
    let origin = origin?;
    if policy.wildcard {
        // Credentials cannot be set here: the pair is rejected in cors_policy.
        if policy.credentials {
            return Some(origin.to_string());
        }
        return Some("*".to_string());
    }
    // Exact equality. Never `ends_with`, never `contains`. Echoing `origin` and
    // echoing the matched entry are the same bytes precisely because the test is
    // equality, so the exactness of the echo follows from the exactness of match.
    if policy.origins.iter().any(|entry| entry.as_str() == origin) {
        return Some(origin.to_string());
    }
    None
}

/// Whether `Vary: Origin` must accompany the decision.
///
/// # Returns
///
/// True unless the response is the same for every origin — that is, unless the
/// policy is a wildcard without credentials, where the emitted `*` does not
/// depend on the request.
pub(super) fn varies(policy: &Policy) -> bool {
    !policy.wildcard || policy.credentials
}
