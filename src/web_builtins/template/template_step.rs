//! One evaluation step over the scanned piece list, plus unsupported-tag rejection.
//!
//! Rejection lives here rather than in the dispatcher so the guidance can grow
//! without pushing that file over the line budget.

use super::template_block::Render;
use super::template_emit::emit;
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
    state: &Render<'_>,
    out: &mut String,
) -> Result<usize, String> {
    match &pieces[index] {
        Piece::Text(text) => {
            out.push_str(text);
            Ok(index + 1)
        }
        // A comment contributes nothing, not even whitespace.
        Piece::Comment => Ok(index + 1),
        Piece::Raw(name) => {
            // `{{{ }}}` is raw regardless of the render's escaping setting.
            out.push_str(&emit(name, context, &state.nested(false, false))?);
            Ok(index + 1)
        }
        Piece::Escaped(name) => {
            out.push_str(&emit(name, context, state)?);
            Ok(index + 1)
        }
        Piece::Tag(body) => super::template_tag::tag(pieces, index, body, context, state, out),
    }
}

/// Reject an unsupported tag keyword.
///
/// Each is named individually, with a hint where an alternative exists, so a ported
/// template fails with a message a reader can act on rather than rendering a silent
/// hole.
///
/// # Errors
///
/// Always returns an error; the return type matches the dispatcher's.
pub(super) fn reject(keyword: &str) -> Result<usize, String> {
    let hint = match keyword {
        "macro" | "endmacro" | "import" => " — define a helper fn in tetherscript instead",
        "set" => " — compute the value in tetherscript and pass it in the context",
        _ => "",
    };
    Err(format!(
        "template: unsupported tag `{keyword}`{hint} (have: if, else, endif, for, endfor, \
         block, endblock, extends, include)"
    ))
}
