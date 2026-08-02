//! `{% for item in items %}` evaluation.
//!
//! Each iteration renders the body against a child context holding the loop
//! variable, so the binding cannot leak past `endfor`.

use super::template_block::{iterable, matching_end, render};
use super::template_scan::Piece;
use crate::value::Value;

/// Render a `for` block, returning the index just past its `endfor`.
///
/// # Errors
///
/// Returns an error for a malformed header, a non-list subject, or an unbalanced
/// block.
pub(super) fn run(
    pieces: &[Piece<'_>],
    index: usize,
    body: &str,
    context: &Value,
    escaping: bool,
    out: &mut String,
) -> Result<usize, String> {
    let (name, subject) = super::template_loop_header::parse(body)?;
    let (end, _) = matching_end(pieces, index)?;
    for item in iterable(context, subject)? {
        let scope = super::template_loop_header::child(context, name, item)?;
        out.push_str(&render(&pieces[index + 1..end], &scope, escaping)?);
    }
    Ok(end + 1)
}
