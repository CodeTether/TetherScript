//! `Accept` header content negotiation.
//!
//! Matching is deliberately structural rather than a substring test: `text/html`
//! must not be considered acceptable merely because it appears inside some longer
//! token. Quality values are parsed only far enough to be ignored, since callers
//! ask a yes/no question rather than requesting a ranked preference.

use std::collections::HashMap;

use super::header_lookup::find;
use crate::value::Value;

/// Decide whether the client accepts `content_type`.
///
/// # Arguments
///
/// * `headers` — Header map.
/// * `content_type` — Candidate type such as `application/json`.
///
/// # Returns
///
/// True when the `Accept` header lists the exact type, its `type/*` wildcard, or
/// `*/*`. An absent or empty `Accept` header means the client expressed no
/// preference, which RFC 9110 treats as accepting anything, so this returns true.
pub(super) fn accepts(headers: &HashMap<String, Value>, content_type: &str) -> bool {
    let Some(header) = find(headers, "accept") else {
        return true;
    };
    if header.trim().is_empty() {
        return true;
    }

    // Compare against the bare type, ignoring any parameters on the candidate.
    let candidate = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim();
    let (candidate_type, candidate_sub) = split_type(candidate);

    header.split(',').any(|entry| {
        // Drop the q-value and any other parameters before comparing.
        let entry = entry.split(';').next().unwrap_or(entry).trim();
        let (entry_type, entry_sub) = split_type(entry);
        match (entry_type, entry_sub) {
            ("*", "*") => true,
            (t, "*") => t.eq_ignore_ascii_case(candidate_type),
            (t, s) => {
                t.eq_ignore_ascii_case(candidate_type) && s.eq_ignore_ascii_case(candidate_sub)
            }
        }
    })
}

/// Split `type/subtype`, treating a bare `*` as `*/*`.
fn split_type(value: &str) -> (&str, &str) {
    match value.split_once('/') {
        Some((left, right)) => (left.trim(), right.trim()),
        None if value == "*" => ("*", "*"),
        None => (value, ""),
    }
}
