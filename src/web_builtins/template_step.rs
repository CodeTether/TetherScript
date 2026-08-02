//! One evaluation step over the scanned piece list.
//!
//! Split from [`super::template_block`] so each file owns one concern.

use super::template_context::lookup;
use super::template_escape::escape;
use super::template_scan::Piece;
use crate::value::Value;

/// Emit the piece at `index`, returning the next index to visit.
///
/// # Errors
///
/// Returns an error for an unknown key, an unbalanced block, or an unknown tag.
pub(super) fn step(
    pieces: &[Piece<'_>],
    index: usize,
    context: &Value,
    escaping: bool,
    out: &mut String,
) -> Result<usize, String> {
    match &pieces[index] {
        Piece::Text(text) => {
            out.push_str(text);
            Ok(index + 1)
        }
        Piece::Raw(name) => {
            out.push_str(&lookup(context, name)?);
            Ok(index + 1)
        }
        Piece::Escaped(name) => {
            let value = lookup(context, name)?;
            if escaping {
                out.push_str(&escape(&value));
            } else {
                out.push_str(&value);
            }
            Ok(index + 1)
        }
        Piece::Tag(body) => super::template_tag::tag(pieces, index, body, context, escaping, out),
    }
}
