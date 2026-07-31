//! Pattern and path segmentation.
//!
//! Both sides of a match are reduced to segment lists first, so the matcher never
//! reasons about raw strings containing `/`. This is what makes a `{name}` capture
//! structurally incapable of spanning a separator.
//!
//! # Trailing slashes
//!
//! Empty segments are dropped, so `/a/b`, `/a/b/`, and `//a//b` all segment to
//! `["a", "b"]`. A trailing slash is therefore **not** significant and
//! `/customers/{id}` matches `/customers/7/`. Actix treats the two as distinct by
//! default and offers `NormalizePath` to merge them; this implementation folds
//! them unconditionally because the port dispatches by hand and a 404 caused by a
//! stray slash is a silent, hard-to-see bug. The choice is asserted in
//! `tests/web_route.rs`.

/// Split a path into its non-empty segments, still percent-encoded.
///
/// # Arguments
///
/// * `path` — Request path or route pattern.
///
/// # Returns
///
/// Segments in order, with empty ones removed.
pub(super) fn split(path: &str) -> Vec<&str> {
    path.split('/').filter(|part| !part.is_empty()).collect()
}

/// A single pattern segment.
#[derive(Debug, PartialEq)]
pub(super) enum Segment<'a> {
    /// Must equal the corresponding path segment exactly.
    Literal(&'a str),
    /// Captures exactly one segment under this name.
    Param(&'a str),
    /// Captures every remaining segment, separators included.
    Tail(&'a str),
}

/// Classify one pattern segment.
///
/// A brace-wrapped segment carrying a regex suffix is treated as a tail capture
/// only for the `.*` form Actix uses for catch-alls; any other regex is rejected
/// by the caller rather than silently mis-matched.
pub(super) fn classify(segment: &str) -> Result<Segment<'_>, String> {
    let Some(inner) = segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')) else {
        if segment.contains('{') || segment.contains('}') {
            return Err(format!(
                "route pattern segment `{segment}` has an unbalanced brace"
            ));
        }
        return Ok(Segment::Literal(segment));
    };
    match inner.split_once(':') {
        None => named(inner).map(Segment::Param),
        Some((name, ".*")) => named(name).map(Segment::Tail),
        Some((_, regex)) => Err(format!(
            "route pattern `{{{inner}}}` uses regex `{regex}`; only `.*` tail captures are supported"
        )),
    }
}

/// Reject an empty capture name so `{}` cannot produce an unnamed parameter.
fn named(name: &str) -> Result<&str, String> {
    if name.is_empty() {
        return Err("route pattern has an empty parameter name `{}`".into());
    }
    Ok(name)
}
