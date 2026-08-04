//! The evaluation loop and per-piece stepping.
//!
//! Inlined `step` from the former `template_step` to save one mod declaration.

use super::template_block::Render;
use super::template_emit::emit;
use super::template_macro_hole::hole;
use super::template_scan::Piece;
use crate::value::Value;

/// Render `pieces` against `context` with the given render state.
///
/// # Errors
///
/// Returns an error for an unknown key, an unbalanced block, or an unsupported tag.
pub(super) fn render_with(
    pieces: &[Piece<'_>],
    context: &Value,
    state: &Render<'_>,
) -> Result<String, String> {
    let mut out = String::new();
    let mut index = 0usize;
    while index < pieces.len() {
        index = step(pieces, index, context, state, &mut out)?;
    }
    Ok(out)
}

fn step(
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
        Piece::Comment => Ok(index + 1),
        Piece::Raw(name) => {
            out.push_str(&emit(name, context, &state.nested(false, false))?);
            Ok(index + 1)
        }
        Piece::Escaped(name) => {
            out.push_str(&hole(pieces, name, context, state)?);
            Ok(index + 1)
        }
        Piece::Tag(body) => super::template_tag::tag(pieces, index, body, context, state, out),
    }
}
