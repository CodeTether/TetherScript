//! Delimiter location and extraction.
//!
//! Split from [`super::template_scan`] so scanning owns the piece structure and
//! this file owns the byte-level delimiter rules.

/// Byte offset of the next `{{` or `{%`, whichever comes first.
pub(super) fn next_delimiter(rest: &str) -> Option<usize> {
    match (rest.find("{{"), rest.find("{%")) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Extract a trimmed body between `open` and `close`, with bytes consumed.
///
/// # Errors
///
/// Returns an error when `close` never appears, or when the body is empty. Both
/// name the delimiter, since a stray `{{` in a large template is otherwise hard to
/// locate.
pub(super) fn delimited<'a>(
    after: &'a str,
    open: &str,
    close: &str,
) -> Result<(&'a str, usize), String> {
    let start = open.len();
    let end = after[start..]
        .find(close)
        .ok_or_else(|| format!("template: unclosed `{open}`"))?;
    let body = after[start..start + end].trim();
    if body.is_empty() {
        return Err(format!("template: empty `{open}{close}`"));
    }
    Ok((body, start + end + close.len()))
}
