//! Pattern matching against a request path.
//!
//! # No match is not an error
//!
//! A router tries many patterns per request, and all but one are expected to
//! fail. So a non-match returns `Ok(None)` and reaches scripts as `nil`, while
//! `Err` is reserved for a malformed *pattern* — a bug in the program, not in the
//! request. Conflating the two would force `?` on every candidate and make the
//! first miss abort dispatch.

use std::collections::HashMap;

use super::route_decode::decode;
use super::route_segments::{classify, split, Segment};
use super::route_tail;

/// Match `path` against `pattern`, capturing named parameters.
///
/// # Arguments
///
/// * `pattern` — Actix-style pattern, for example `/customers/{id}`.
/// * `path` — Request path.
///
/// # Returns
///
/// `Ok(Some(captures))` on a match, or `Ok(None)` when the pattern simply does
/// not apply.
///
/// # Errors
///
/// Returns an error when `pattern` is malformed: an unbalanced brace, an empty
/// name, an unsupported regex, or a tail capture that is not the final segment.
pub(super) fn match_path(
    pattern: &str,
    path: &str,
) -> Result<Option<HashMap<String, String>>, String> {
    let pattern_segments = split(pattern);
    let path_segments = split(path);
    let mut captures = HashMap::new();

    for (index, raw) in pattern_segments.iter().enumerate() {
        match classify(raw)? {
            Segment::Tail(name) => {
                if index + 1 != pattern_segments.len() {
                    return Err(format!(
                        "route pattern `{pattern}`: tail capture `{{{name}:.*}}` must be last"
                    ));
                }
                let (name, value) = route_tail::capture(name, &path_segments, index)?;
                captures.insert(name, value);
                return Ok(Some(captures));
            }
            Segment::Param(name) => match path_segments.get(index) {
                Some(segment) => {
                    captures.insert(name.to_string(), decode(segment)?);
                }
                None => return Ok(None),
            },
            Segment::Literal(text) => match path_segments.get(index) {
                Some(segment) if segment == &text => {}
                _ => return Ok(None),
            },
        }
    }

    // Without a tail capture the lengths must agree, or `/a` would match `/a/b`.
    if path_segments.len() != pattern_segments.len() {
        return Ok(None);
    }
    Ok(Some(captures))
}

/// List the parameter names a pattern declares, in order.
///
/// # Arguments
///
/// * `pattern` — Actix-style pattern.
///
/// # Returns
///
/// Names without braces or regex suffixes; empty when the pattern is literal.
///
/// # Errors
///
/// Returns an error when a segment is malformed, matching [`match_path`].
pub(super) fn params(pattern: &str) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    for raw in split(pattern) {
        match classify(raw)? {
            Segment::Param(name) | Segment::Tail(name) => names.push(name.to_string()),
            Segment::Literal(_) => {}
        }
    }
    Ok(names)
}
