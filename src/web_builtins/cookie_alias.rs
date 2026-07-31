//! Option-name aliasing for the cookie options map.
//!
//! Scripts write snake_case (`http_only`, `max_age`), but the `Set-Cookie` header
//! spells the same attributes `HttpOnly` and `Max-Age`. Accepting both spellings
//! avoids a silent no-op when a caller uses header casing, which for an attribute
//! like `HttpOnly` would quietly weaken a session cookie.

use std::collections::HashMap;

use crate::value::Value;

/// Find an option by its canonical name or any accepted alias.
///
/// # Arguments
///
/// * `opts` — Script-supplied options map.
/// * `key` — Canonical snake_case option name.
///
/// # Returns
///
/// A reference to the first matching value, or `None` when no spelling is present.
pub(super) fn lookup<'a>(opts: &'a HashMap<String, Value>, key: &str) -> Option<&'a Value> {
    if let Some(value) = opts.get(key) {
        return Some(value);
    }
    aliases(key).iter().find_map(|alias| opts.get(*alias))
}

/// Accepted alternative spellings for each canonical option name.
fn aliases(key: &str) -> &'static [&'static str] {
    match key {
        "http_only" => &["httpOnly", "HttpOnly", "httponly"],
        "same_site" => &["sameSite", "SameSite", "samesite"],
        "max_age" => &["maxAge", "Max-Age", "max-age", "maxage"],
        "path" => &["Path"],
        "domain" => &["Domain"],
        "secure" => &["Secure"],
        "expires" => &["Expires"],
        _ => &[],
    }
}

/// Canonicalize a `SameSite` value to its header spelling.
///
/// # Arguments
///
/// * `value` — Script-supplied value, in any casing.
///
/// # Returns
///
/// `"Strict"`, `"Lax"`, or `"None"`.
///
/// # Errors
///
/// Returns an error naming the rejected value. An unrecognized `SameSite` is not
/// silently dropped: emitting the attribute with a bogus value makes browsers
/// ignore it entirely, which would quietly weaken CSRF protection.
pub(super) fn same_site(value: &str) -> Result<&'static str, String> {
    match value.to_ascii_lowercase().as_str() {
        "strict" => Ok("Strict"),
        "lax" => Ok("Lax"),
        "none" => Ok("None"),
        _ => Err(format!(
            "cookie option `same_site` must be Strict, Lax, or None; got `{value}`"
        )),
    }
}
