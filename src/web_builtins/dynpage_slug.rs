//! Slug normalisation and validation.
//!
//! # Normalisation, in order
//!
//! 1. Strip leading and trailing `/`, so `/about`, `about/`, and `/about/` all
//!    yield `about`. That mirrors `route_segments.rs`, which drops empty segments,
//!    so the two agree that a trailing slash is not significant.
//! 2. Lowercase using ASCII case folding only. Full Unicode folding is avoided
//!    deliberately: it is locale-sensitive (Turkish dotless `i`) and it maps exotic
//!    codepoints onto ASCII (`K` U+212A folds to `k`), which would let two visibly
//!    different requests collapse onto one cache key. Non-ASCII simply fails the
//!    charset instead.
//! 3. Validate against the allowlist in [`super::dynpage_charset`].
//!
//! Nothing is ever removed to *make* a slug valid — see [`super::dynpage_reject`]
//! for why sanitising is the weaker choice.
//!
//! # One segment only
//!
//! `slug_parse` accepts a single segment. `/blog/post` is rejected, because an
//! interior `/` is not in the charset. A multi-segment route is `route.rs`'s job:
//! the caller matches `/blog/{slug}` with `route_match`, takes the decoded capture,
//! and passes that here. That division is what keeps a slug structurally incapable
//! of spanning a separator.

use super::dynpage_charset as charset;
use super::dynpage_reject as reject;

/// Normalise a request path into a slug candidate.
///
/// # Arguments
///
/// * `path` — Raw request path, or a single decoded path segment.
///
/// # Returns
///
/// The trimmed, ASCII-lowercased candidate. It is not yet known to be valid.
fn normalise(path: &str) -> String {
    path.trim_matches('/').to_ascii_lowercase()
}

/// Normalise and validate a request path into a slug.
///
/// # Arguments
///
/// * `path` — Raw request path, or a single decoded path segment.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// The normalised slug.
///
/// # Errors
///
/// Returns an error when the result is empty, longer than the 200-byte limit, or
/// contains anything outside `[a-z0-9_-]`. That covers `..`, `/`, `\`, NUL, and
/// every percent-encoded form of them, since neither `%` nor `.` is a member.
pub(super) fn parse(path: &str, label: &str) -> Result<String, String> {
    let slug = normalise(path);
    reject::check(&slug, label)?;
    Ok(slug)
}

/// Test a slug without normalising it.
///
/// # Arguments
///
/// * `slug` — Candidate to test, exactly as the caller holds it.
///
/// # Returns
///
/// True when the candidate already satisfies the charset and the length limit. An
/// uppercase candidate is **false**, not normalised: `slug_valid` answers "is this
/// usable as-is", and `slug_parse` is the function that transforms.
pub(super) fn valid(slug: &str) -> bool {
    charset::valid(slug)
}
