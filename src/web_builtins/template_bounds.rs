//! Block boundary discovery.
//!
//! Finding the matching close tag is depth-tracked, so an inner `if` inside an
//! outer `if` does not terminate the outer block early.

use super::template_scan::Piece;

/// Find the index of the tag closing the block opened at `open`, and its `else`.
///
/// # Arguments
///
/// * `pieces` — Scanned template.
/// * `open` — Index of the opening `if` or `for` tag.
///
/// # Returns
///
/// The closing tag's index, plus the index of the block's own `else` when present.
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
            // `block` counts too: a block inside an `if` must not let the `if`'s
            // `endif` search stop at the block's `endblock`, and vice versa.
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
