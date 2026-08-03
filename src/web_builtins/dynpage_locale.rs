//! Locale negotiation against a supported list.
//!
//! # Matching, in order
//!
//! For each parsed range, highest quality first
//! ([`super::dynpage_locale_parse`]):
//!
//! 1. Exact match on the full tag: `en-gb` selects supported `en-gb`.
//! 2. Prefix match on the primary subtag: `en-gb` selects supported `en`, and `en`
//!    selects supported `en-us`. Serving a closely related dialect beats falling
//!    back to a language the visitor does not read.
//! 3. `*` selects the first supported locale.
//!
//! The first range that matches anything wins, so a lower-quality range can never
//! outrank a higher one.
//!
//! # Default
//!
//! When the header is absent, empty, or names nothing supported, the **first** entry
//! of `supported` is returned. The caller therefore controls the default by ordering
//! its own list, and this function never returns a locale the caller did not
//! declare. That is the property keeping an attacker-controlled header out of the
//! cache key: the result is always an element of `supported`.

use std::collections::HashMap;

use super::dynpage_locale_parse::{Entry, parse};
use super::dynpage_request::find;
use crate::value::Value;

/// Negotiate the best supported locale for a request.
///
/// # Arguments
///
/// * `headers` — Request header map.
/// * `supported` — Non-empty, caller-declared locales, preferred default first.
///
/// # Returns
///
/// An element of `supported`: the best `Accept-Language` match, or `supported[0]`
/// when the header is absent or matches nothing.
///
/// # Panics
///
/// Panics if `supported` is empty. The caller guarantees non-emptiness — the script
/// surface rejects an empty list in `dynpage_args::str_list_arg` — because there
/// would be no default to fall back to.
pub(super) fn negotiate(headers: &HashMap<String, Value>, supported: &[String]) -> String {
    let default = supported[0].clone();
    let Some(header) = find(headers, "accept-language") else {
        return default;
    };
    parse(&header)
        .iter()
        .find_map(|range| best(range, supported))
        .unwrap_or(default)
}

/// Resolve one language range against the supported list.
fn best(range: &Entry, supported: &[String]) -> Option<String> {
    if range.tag == "*" {
        return supported.first().cloned();
    }
    let wanted = range.tag.as_str();
    let primary = subtag(wanted);
    let exact = supported.iter().find(|have| have.eq_ignore_ascii_case(wanted));
    match exact {
        Some(found) => Some(found.clone()),
        None => supported
            .iter()
            .find(|have| subtag(have.as_str()).eq_ignore_ascii_case(primary))
            .cloned(),
    }
}

/// The primary subtag of a language tag: `en-gb` -> `en`.
fn subtag(tag: &str) -> &str {
    tag.split('-').next().unwrap_or(tag)
}
