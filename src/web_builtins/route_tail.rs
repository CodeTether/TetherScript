//! Tail-capture handling for `{name:.*}`.
//!
//! Split from [`super::route_match`] to keep each file within the line budget.
//! A tail capture is the only construct allowed to span `/`, so it is isolated
//! here where that exception is obvious.

use super::route_decode::decode;

/// Capture every remaining path segment, separators included.
///
/// # Arguments
///
/// * `name` — Capture name, already validated as non-empty.
/// * `path_segments` — All segments of the request path.
/// * `index` — Position of the tail pattern segment.
///
/// # Returns
///
/// The remainder rejoined with `/`. An exhausted path yields an empty string, so
/// `/files/{rest:.*}` still matches `/files` with `rest` empty — matching Actix,
/// where a catch-all may capture nothing.
///
/// # Errors
///
/// Returns an error when a remaining segment carries a malformed percent-escape.
pub(super) fn capture(
    name: &str,
    path_segments: &[&str],
    index: usize,
) -> Result<(String, String), String> {
    let start = index.min(path_segments.len());
    let mut parts = Vec::new();
    for segment in &path_segments[start..] {
        parts.push(decode(segment)?);
    }
    Ok((name.to_string(), parts.join("/")))
}
