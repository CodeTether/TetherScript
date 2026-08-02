//! `{% ... %}` tag dispatch.
//!
//! Split from [`super::template_step`] so expression emission and statement
//! evaluation stay separate.

use std::collections::HashMap;

use super::template_block::{condition, matching_end, render_with};
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
    overrides: &HashMap<String, String>,
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
            out.push_str(&render_with(taken, context, escaping, overrides)?);
            Ok(end + 1)
        }
        "for" => super::template_loop::run(pieces, index, body, context, escaping, overrides, out),
        "block" => {
            super::template_block_tag::run(pieces, index, body, context, escaping, overrides, out)
        }
        // The root template no longer carries `extends`; reaching one means it was
        // not the first tag, which inheritance resolution rejects.
        "extends" => Err("template: `extends` must be the first tag in a template".into()),
        // These are consumed by their opening tag, so reaching one here means it had
        // no opener.
        keyword @ ("else" | "endif" | "endfor" | "endblock") => {
            Err(format!("template: `{keyword}` without a matching opener"))
        }
        other => super::template_tag_unknown::reject(other),
    }
}
