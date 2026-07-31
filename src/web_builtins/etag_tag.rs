//! ETag computation and comparison.
//!
//! Split from [`super::etag`] so hashing and header parsing stay separate from
//! registration. Validators are derived from SHA-256 of the body, so identical
//! bytes always produce the same entity tag and different bytes effectively
//! never collide.

use crate::system::{hex_encode, sha256};

/// Build a strong entity tag, quoted per RFC 9110 section 8.8.3.
///
/// # Arguments
///
/// * `body` — Response body bytes.
///
/// # Returns
///
/// The quoted validator, for example `"9f86d081…"`. The quotes are part of the
/// header value, not decoration: an unquoted tag is malformed.
pub(super) fn strong(body: &[u8]) -> String {
    format!("\"{}\"", hex_encode(&sha256(body)))
}

/// Build a weak validator by prefixing `W/` to the strong form.
///
/// # Arguments
///
/// * `body` — Response body bytes.
///
/// # Returns
///
/// For example `W/"9f86d081…"`. Weak tags assert semantic rather than
/// byte-for-byte equivalence, which is what a cache needs when a response is
/// recompressed or re-rendered without changing meaning.
pub(super) fn weak(body: &[u8]) -> String {
    format!("W/{}", strong(body))
}

/// Strip a `W/` prefix and surrounding quotes, yielding the bare opaque value.
///
/// Comparison is done on this normalized form so `W/"x"` and `"x"` match, per the
/// weak comparison function in RFC 9110 section 8.8.3.2.
fn opaque(tag: &str) -> &str {
    let tag = tag.trim();
    let tag = tag.strip_prefix("W/").unwrap_or(tag);
    tag.trim_matches('"')
}

/// Test an `If-None-Match` header against a candidate entity tag.
///
/// # Arguments
///
/// * `header` — Raw `If-None-Match` value: `*`, one tag, or a comma-separated
///   list, with optional surrounding whitespace.
/// * `etag` — The current validator, strong or weak.
///
/// # Returns
///
/// True when the header matches, meaning the caller should answer `304`.
///
/// Comparison is per-entry and exact on the opaque value — never a substring
/// test. `"abc"` must not match `"abcdef"`, because treating a prefix as a hit
/// serves stale content, which is worse than not caching at all.
pub(super) fn matches(header: &str, etag: &str) -> bool {
    let header = header.trim();
    if header == "*" {
        return true;
    }
    let candidate = opaque(etag);
    header
        .split(',')
        .any(|entry| !entry.trim().is_empty() && opaque(entry) == candidate)
}
