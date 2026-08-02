//! Delimiter location, extraction, and block-boundary discovery.
//!
//! All three are byte-level structural concerns over the same token stream, so they
//! share a file rather than fragmenting into three.

use super::template_scan::Piece;

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
/// Returns an error when `close` never appears, or when the body is empty. Both name
/// the delimiter, since a stray `{{` in a large template is otherwise hard to find.
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

/// Find the tag closing the block opened at `open`, and its `else`.
///
/// Depth-tracked, so an inner block does not terminate an outer one early. `block`
/// counts alongside `if`/`for`: a block inside an `if` must not let the `if`'s search
/// stop at the block's `endblock`.
///
/// # Errors
///
/// Returns an error when the block is never closed.
pub(super) fn matching_end(
    pieces: &[Piece<'_>],
    open: usize,
) -> Result<(usize, Option<usize>), String> {
    let mut depth = 0usize;
    let mut alternate = None;
    for (offset, piece) in pieces.iter().enumerate().skip(open) {
        let Piece::Tag(body) = piece else { continue };
        match body.split_whitespace().next().unwrap_or("") {
            "if" | "for" | "block" => depth += 1,
            // Only the outermost `else` at depth 1 belongs to this block.
            "else" if depth == 1 && alternate.is_none() => alternate = Some(offset),
            "endif" | "endfor" | "endblock" => {
                depth -= 1;
                if depth == 0 {
                    return Ok((offset, alternate));
                }
            }
            _ => {}
        }
    }
    Err("template: unbalanced block; missing `endif`, `endfor`, or `endblock`".to_string())
}
