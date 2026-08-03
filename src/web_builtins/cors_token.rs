//! Token normalization for methods and header names.
//!
//! HTTP methods are case-sensitive and conventionally uppercase (RFC 9110 §9),
//! while header *names* are case-insensitive (§5.1). Those two rules differ, so
//! they are normalized differently and deliberately: methods are upper-cased so
//! `get` in a config still matches a real `GET`, and header names are lower-cased
//! so `Content-Type` on the wire matches `content-type` in the allow-list.
//!
//! Origins are **not** normalized here. An origin is compared byte for byte, so
//! folding its case would be a silent policy widening.

/// Normalize a method token for comparison.
///
/// # Arguments
///
/// * `method` — A method as written in a config or request header.
///
/// # Returns
///
/// The trimmed, upper-cased token.
pub(super) fn method(method: &str) -> String {
    method.trim().to_ascii_uppercase()
}

/// Normalize a header name for comparison.
///
/// # Arguments
///
/// * `name` — A header name in any casing.
///
/// # Returns
///
/// The trimmed, lower-cased name.
pub(super) fn header(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

/// Split a comma-separated header list into normalized header names.
///
/// # Arguments
///
/// * `list` — Raw value of `Access-Control-Request-Headers`.
///
/// # Returns
///
/// One lower-cased name per non-empty entry. An all-whitespace entry is dropped
/// rather than becoming an empty name that could never match the allow-list.
pub(super) fn header_list(list: &str) -> Vec<String> {
    list.split(',')
        .map(header)
        .filter(|name| !name.is_empty())
        .collect()
}
