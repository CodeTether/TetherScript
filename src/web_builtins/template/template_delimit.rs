//! Delimiter location, extraction, and block-structure discovery.
//!
//! All byte- and index-level structural concerns over the same token stream: finding
//! delimiters, extracting their bodies, and locating the branches and end of a block.

use super::template_scan::Piece;

/// Byte offset of the next `{{`, `{%`, or `{#`, whichever comes first.
pub(super) fn next_delimiter(rest: &str) -> Option<usize> {
    [rest.find("{{"), rest.find("{%"), rest.find("{#")]
        .into_iter()
        .flatten()
        .min()
}

/// Extract a trimmed body between `open` and `close`, with bytes consumed.
///
/// # Errors
///
/// Returns an error when `close` never appears, or when the body is empty. Both name the
/// delimiter, since a stray `{{` in a large template is otherwise hard to find.
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

/// One conditional branch: its condition key, and the tag that opened it.
pub(super) struct Branch<'a> {
    /// Condition key, or `None` for the final `else`.
    pub condition: Option<&'a str>,
    /// Index of the tag that opened this branch.
    pub at: usize,
}

/// Find the index of the tag closing the block opened at `open`.
///
/// Depth-tracked, so an inner block does not terminate an outer one early. `block`
/// counts alongside `if`/`for`: a block inside an `if` must not let the `if`'s search
/// stop at the block's `endblock`.
///
/// # Errors
///
/// Returns an error when the block is never closed.
pub(super) fn matching_end(pieces: &[Piece<'_>], open: usize) -> Result<usize, String> {
    let mut depth = 0usize;
    for (offset, piece) in pieces.iter().enumerate().skip(open) {
        let Piece::Tag(body) = piece else { continue };
        match body.split_whitespace().next().unwrap_or("") {
            "if" | "for" | "block" => depth += 1,
            "endif" | "endfor" | "endblock" => {
                depth -= 1;
                if depth == 0 {
                    return Ok(offset);
                }
            }
            _ => {}
        }
    }
    Err("template: unbalanced block; missing `endif`, `endfor`, or `endblock`".to_string())
}
