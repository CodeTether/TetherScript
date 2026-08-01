//! Hand-written predicate scanners for common input shapes.
//!
//! There is no regex engine in the default build, so each check is a single pass
//! over the characters. That is a feature here, not a workaround: a scanner has
//! no catastrophic-backtracking failure mode, which matters when the input comes
//! from an untrusted form post.

/// Pragmatic email shape check.
///
/// # Arguments
///
/// * `text` — Candidate address.
///
/// # Returns
///
/// True when `text` has exactly one `@`, a non-empty local part, and a dotted
/// domain whose final label is at least two characters, with no whitespace and no
/// consecutive dots.
///
/// # Scope
///
/// This is a **filter, not proof of deliverability**. It deliberately does not
/// implement the RFC 5322 grammar: quoted local parts, comments, and bracketed
/// address literals are all rejected even though they are legal. The only way to
/// know an address accepts mail is to send to it.
pub(super) fn is_email(text: &str) -> bool {
    if text.contains(char::is_whitespace) || text.contains("..") {
        return false;
    }
    let Some((local, domain)) = text.split_once('@') else {
        return false;
    };
    // A second `@` lands in `domain`, so rejecting it here enforces "exactly one".
    if local.is_empty() || domain.contains('@') {
        return false;
    }
    let labels: Vec<&str> = domain.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|label| !label.is_empty())
        && labels.last().is_some_and(|tld| tld.len() >= 2)
}

/// Check a URL slug: lowercase alphanumerics separated by single hyphens.
///
/// # Arguments
///
/// * `text` — Candidate slug.
///
/// # Returns
///
/// True when `text` is non-empty, contains only `a-z`, `0-9`, and `-`, has no
/// leading, trailing, or doubled hyphen. Uppercase is rejected rather than
/// folded, because two slugs differing only in case would collide as URLs.
pub(super) fn is_slug(text: &str) -> bool {
    if text.is_empty() || text.starts_with('-') || text.ends_with('-') {
        return false;
    }
    if text.contains("--") {
        return false;
    }
    text.chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

/// Check that every character is an ASCII digit.
///
/// # Arguments
///
/// * `text` — Candidate digit string.
///
/// # Returns
///
/// True when `text` is non-empty and every character is `0`-`9`. Non-ASCII
/// decimal digits such as `٣` are rejected: they are digits to Unicode but not
/// parseable by anything downstream that expects ASCII.
pub(super) fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|ch| ch.is_ascii_digit())
}
