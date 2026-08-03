//! `{% ... %}` tag dispatch.
//!
//! Split from [`super::template_step`] so expression emission and statement
//! evaluation stay separate.

use super::template_block::{render_with, Render};
use super::template_branch::branches;
use super::template_condition::evaluate;
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
    state: &Render<'_>,
    out: &mut String,
) -> Result<usize, String> {
    let mut words = body.split_whitespace();
    match words.next().unwrap_or("") {
        "if" => conditional(pieces, index, context, state, out),
        "for" => super::template_loop::run(pieces, index, body, context, state, out),
        "block" => super::template_blocks::run(pieces, index, body, context, state, out),
        "include" => {
            super::template_include::run(body, context, state, out)?;
            Ok(index + 1)
        }
        // The root template no longer carries `extends`; reaching one means it was not
        // the first tag, which inheritance resolution rejects.
        "extends" => Err("template: `extends` must be the first tag in a template".into()),
        // These are consumed by their opening tag, so reaching one here means it had
        // no opener.
        keyword @ ("else" | "elif" | "endif" | "endfor" | "endblock") => {
            Err(format!("template: `{keyword}` without a matching opener"))
        }
        other => super::template_step::reject(other),
    }
}

/// Render the first satisfied branch of an `if`/`elif`/`else` chain.
///
/// Only the taken branch is evaluated, so an untaken branch may reference keys that do
/// not exist — which is exactly how a view guards an optional value.
///
/// # Errors
///
/// Returns an error for an unbalanced block or a failure rendering the taken branch.
fn conditional(
    pieces: &[Piece<'_>],
    index: usize,
    context: &Value,
    state: &Render<'_>,
    out: &mut String,
) -> Result<usize, String> {
    let (found, end) = branches(pieces, index)?;
    for (position, branch) in found.iter().enumerate() {
        let taken = match branch.condition {
            Some(key) => evaluate(context, key)?,
            // `else` always matches; reaching it means every earlier test failed.
            None => true,
        };
        if taken {
            // The branch body runs to the next branch tag, or to `endif` when last.
            let stop = found.get(position + 1).map_or(end, |next| next.at);
            out.push_str(&render_with(&pieces[branch.at + 1..stop], context, state)?);
            break;
        }
    }
    Ok(end + 1)
}
