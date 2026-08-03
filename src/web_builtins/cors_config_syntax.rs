//! Exact-origin syntax validation for the allow-list.
//!
//! An origin is a scheme, a host, and an optional port — nothing else (RFC 6454
//! §6.2). A config entry carrying a path, a trailing slash, or a query is a sign
//! the author believed CORS matched URLs, and it can never equal the `Origin`
//! header a browser sends, so it is rejected loudly instead of silently never
//! matching.
//!
//! # Security
//!
//! Rejecting `*` as a *list entry* matters: a caller who writes `["*"]` expecting
//! a wildcard would otherwise get an allow-list containing the literal string
//! `"*"`, which no browser ever sends, and would conclude CORS is broken. The
//! wildcard is only ever the bare string `"*"` in place of the list, so the one
//! dangerous setting has exactly one spelling and is easy to grep for.

/// Build the rejection message for one malformed origin.
fn bad(origin: &str, why: &str) -> Result<(), String> {
    Err(format!("cors_policy: origin `{origin}` {why}"))
}

/// Whether `scheme` is a syntactically valid URI scheme (RFC 3986 §3.1).
fn scheme_ok(scheme: &str) -> bool {
    !scheme.is_empty()
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '+' || c == '.')
}

/// Validate one allow-list origin.
///
/// # Arguments
///
/// * `origin` — A candidate origin such as `https://app.example.com:8443`.
///
/// # Returns
///
/// `Ok(())` when the value is a bare scheme/host/port origin.
///
/// # Errors
///
/// Returns an error naming `origin` and the specific defect: a wildcard entry,
/// embedded whitespace, a missing or malformed scheme, an empty host, or a path,
/// query, or trailing slash.
pub(super) fn check(origin: &str) -> Result<(), String> {
    if origin == "*" {
        return bad(
            origin,
            "must not appear inside the list; pass the string \"*\" as `origins` instead",
        );
    }
    if origin.contains(char::is_whitespace) {
        return bad(origin, "must not contain whitespace");
    }
    let Some((scheme, rest)) = origin.split_once("://") else {
        return bad(origin, "must include a scheme, as in https://example.com");
    };
    if !scheme_ok(scheme) {
        return bad(origin, "has an invalid scheme");
    }
    if rest.is_empty() {
        return bad(origin, "has an empty host");
    }
    if rest.contains('/') || rest.contains('?') || rest.contains('#') {
        return bad(
            origin,
            "must be scheme://host[:port] with no path, query, or trailing slash",
        );
    }
    Ok(())
}
