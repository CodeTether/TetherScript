//! `{% if %}` / `{% elif %}` / `{% else %}` conditional rendering.
//!
//! Split from [`super::template_tag`] so each file stays within the line budget.

use super::template_block::{render_with, Render};
use super::template_branch::branches;
use super::template_condition::evaluate;
use super::template_scan::Piece;
use crate::value::Value;

/// Render the first satisfied branch of an `if`/`elif`/`else` chain.
///
/// Only the taken branch is evaluated, so an untaken branch may reference keys that do not exist.
///
/// # Errors
///
/// Returns an error for an unbalanced block or a failure rendering the taken branch.
pub(super) fn conditional(
    pieces: &[Piece<'_>], index: usize, context: &Value,
    state: &Render<'_>, out: &mut String,
) -> Result<usize, String> {
    let (found, end) = branches(pieces, index)?;
    for (position, branch) in found.iter().enumerate() {
        let taken = match branch.condition {
            Some(key) => evaluate(context, key)?,
            None => true,
        };
        if taken {
            let stop = found.get(position + 1).map_or(end, |next| next.at);
            out.push_str(&render_with(&pieces[branch.at + 1..stop], context, state)?);
            break;
        }
    }
    Ok(end + 1)
}
