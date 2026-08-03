//! Why a candidate slug was refused.
//!
//! # Reject, never sanitise
//!
//! The tempting alternative is to strip the dangerous characters and carry on.
//! That is the weaker choice, for three concrete reasons.
//!
//! 1. **Stripping manufactures valid input.** `..%2f..%2fetc` with `.` and `/`
//!    deleted becomes `etc`, a slug that resolves to a *real* page the attacker
//!    never asked for. The request now succeeds, silently, against the wrong
//!    resource — worse than a 404.
//! 2. **Stripping is order-dependent and so unauditable.** `....//` loses one
//!    `../` and yields `../`, so a sanitiser must be iterated to a fixed point,
//!    and "did it converge" is far harder to review than "does it match the
//!    allowlist".
//! 3. **The caller loses the signal.** A slug containing `/` is either a router
//!    bug or an attack. Both deserve an error a log can show; sanitising turns
//!    both into an ordinary lookup nobody ever notices.
//!
//! So this module only ever explains a rejection. It never repairs one.
//!
//! The offending value is echoed truncated, because the input is
//! attacker-controlled and an unbounded echo turns one request into an unbounded
//! log line.

use super::dynpage_charset as charset;

/// Characters of the offending slug included in an error message.
const PREVIEW: usize = 48;

/// Forbidden characters, each with the reason a reader needs.
const FORBIDDEN: [(char, &str); 4] = [
    ('%', "must not contain a percent escape"),
    ('\0', "must not contain a NUL byte"),
    ('/', "must not contain a path separator `/`"),
    ('\\', "must not contain a backslash"),
];

/// Diagnose a normalised slug, naming the specific reason it is unusable.
///
/// # Arguments
///
/// * `slug` — Normalised (trimmed, lowercased) candidate.
/// * `label` — Built-in name used in the error message.
///
/// # Returns
///
/// `Ok(())` when the slug is acceptable.
///
/// # Errors
///
/// Returns an error naming the offending construct: a percent escape, a NUL, a
/// `/`, a `\`, a `..` traversal, an over-long value together with its limit, or a
/// character outside the charset.
pub(super) fn check(slug: &str, label: &str) -> Result<(), String> {
    if slug.is_empty() {
        return Err(format!("{label}: slug must not be empty"));
    }
    let shown = preview(slug);
    if let Some(reason) = forbidden(slug) {
        return Err(format!("{label}: slug `{shown}` {reason}"));
    }
    if slug.len() > charset::MAX_LEN {
        let (seen, limit) = (slug.len(), charset::MAX_LEN);
        return Err(format!(
            "{label}: slug is {seen} bytes, over the {limit}-byte limit"
        ));
    }
    if !charset::valid(slug) {
        return Err(format!(
            "{label}: slug `{shown}` must match [a-z0-9_-]+ only"
        ));
    }
    Ok(())
}

/// Name the first forbidden construct in `slug`, if any.
///
/// A percent sign is forbidden outright rather than decoded: percent-decoding is
/// `route_decode.rs`'s job and runs *after* segmentation. Decoding again here
/// would let `%2F` become a real separator inside what the caller believes is one
/// segment. `..` is checked explicitly so a traversal attempt is reported as a
/// traversal, even though `.` is already outside the charset.
fn forbidden(slug: &str) -> Option<&'static str> {
    for (bad, reason) in FORBIDDEN {
        if slug.contains(bad) {
            return Some(reason);
        }
    }
    slug.contains("..")
        .then_some("must not contain a `..` traversal")
}

/// Truncate at a character boundary for inclusion in an error message.
fn preview(slug: &str) -> &str {
    match slug.char_indices().nth(PREVIEW) {
        Some((index, _)) => &slug[..index],
        None => slug,
    }
}
