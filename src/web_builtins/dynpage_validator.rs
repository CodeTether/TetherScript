//! `If-None-Match` validator comparison.
//!
//! # Why this is not a call into `etag.rs`
//!
//! `etag_tag::matches` implements the same rule, but it is `pub(super)` within the
//! `etag` group and therefore unreachable from this one. Widening it would mean
//! editing a file this change does not own, so the rule is restated here with the
//! same semantics, and `tests/web_dynpage.rs` asserts the behaviour so the two
//! cannot drift silently. At the script level a caller still composes the groups:
//! the tag handed to `page_not_modified` is the one `etag_of` produced.
//!
//! # Comparison is per-entry and exact
//!
//! Each `If-None-Match` entry is trimmed, then stripped of a `W/` prefix and of
//! surrounding quotes, and compared for equality against the same normalisation of
//! the cached tag — the weak comparison function of RFC 9110 §8.8.3.2. It is
//! deliberately **not** a substring test: if `"abc"` matched `"abcdef"` the server
//! would answer 304 for a body the client has never seen, which is worse than not
//! caching at all.

/// Strip a `W/` prefix and surrounding quotes, yielding the bare opaque value.
fn opaque(tag: &str) -> &str {
    let tag = tag.trim();
    let tag = tag.strip_prefix("W/").unwrap_or(tag);
    tag.trim_matches('"')
}

/// Test an `If-None-Match` header against a cached entity tag.
///
/// # Arguments
///
/// * `header` — Raw `If-None-Match` value: `*`, one tag, or a comma-separated
///   list.
/// * `etag` — The cached validator, strong or weak.
///
/// # Returns
///
/// True when the client already holds this representation, meaning the caller
/// should answer 304. An empty cached tag never matches: there is nothing to
/// compare, and matching would claim freshness the server cannot vouch for.
pub(super) fn matches(header: &str, etag: &str) -> bool {
    let candidate = opaque(etag);
    if candidate.is_empty() {
        return false;
    }
    let header = header.trim();
    if header == "*" {
        return true;
    }
    header
        .split(',')
        .any(|entry| !entry.trim().is_empty() && opaque(entry) == candidate)
}
