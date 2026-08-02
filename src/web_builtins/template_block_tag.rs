//! `{% block name %}` rendering.
//!
//! When an override exists for the name, its source is scanned and rendered in
//! place of the parent's body. Otherwise the parent's own body renders, which is
//! what makes a block's content a default rather than a requirement.

use std::collections::HashMap;

use super::template_block::render_with;
use super::template_blocks::name_of;
use super::template_bounds::matching_end;
use super::template_scan::{scan, Piece};
use crate::value::Value;

/// Render a `block` tag, returning the index just past its `endblock`.
///
/// # Errors
///
/// Returns an error for an unnamed block, an unbalanced `endblock`, or any failure
/// rendering the chosen body.
pub(super) fn run(
    pieces: &[Piece<'_>],
    index: usize,
    body: &str,
    context: &Value,
    escaping: bool,
    overrides: &HashMap<String, String>,
    out: &mut String,
) -> Result<usize, String> {
    let name = name_of(body)?;
    let (end, _) = matching_end(pieces, index)?;
    match overrides.get(name) {
        Some(source) => {
            // The override is re-scanned here rather than at collection time so a
            // block body may itself contain blocks, ifs, and loops.
            let replacement = scan(source)?;
            out.push_str(&render_with(&replacement, context, escaping, overrides)?);
        }
        None => {
            let default = &pieces[index + 1..end];
            out.push_str(&render_with(default, context, escaping, overrides)?);
        }
    }
    Ok(end + 1)
}
