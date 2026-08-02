//! `{% ... %}` tag dispatch.
//!
//! Split from [`super::template_step`] so expression emission and statement
//! evaluation stay separate.

use super::template_block::{condition, matching_end, render};
use super::template_scan::Piece;
use crate::value::Value;

/// Evaluate one tag, returning the next index to visit.
///
/// # Errors
///
/// Returns an error for a malformed header, an unbalanced block, or an unknown
/// keyword.
pub(super) fn tag(
    pieces: &[Piece<'_>],
    index: usize,
    body: &str,
    context: &Value,
    escaping: bool,
    out: &mut String,
) -> Result<usize, String> {
    let mut words = body.split_whitespace();
    match words.next().unwrap_or("") {
        "if" => {
            let key = words.next().ok_or("template: `if` needs a condition")?;
            let (end, alternate) = matching_end(pieces, index)?;
            let taken = if condition(context, key)? {
                &pieces[index + 1..alternate.unwrap_or(end)]
            } else {
                alternate.map_or(&pieces[0..0], |at| &pieces[at + 1..end])
            };
            out.push_str(&render(taken, context, escaping)?);
            Ok(end + 1)
        }
        "for" => super::template_loop::run(pieces, index, body, context, escaping, out),
        // These are consumed by their opening tag, so reaching one here means it had
        // no opener.
        keyword @ ("else" | "endif" | "endfor") => {
            Err(format!("template: `{keyword}` without a matching opener"))
        }
        other => Err(format!(
            "template: unsupported tag `{other}` (have: if, else, endif, for, endfor)"
        )),
    }
}
