//! Statement evaluation for the block-aware template subset.
//!
//! Evaluation walks the piece list with an explicit index rather than recursing
//! over slices, which keeps nesting depth independent of the Rust stack.
//!
//! Block overrides are carried through every layer because a `{% block %}` may
//! appear inside an `{% if %}` or `{% for %}` in the parent template.

use std::collections::HashMap;

use super::template_scan::Piece;
use crate::value::Value;

/// Render `pieces`, substituting `overrides` for matching `{% block %}` bodies.
///
/// # Errors
///
/// Returns an error for an unknown key, an unbalanced block, or an unsupported tag.
pub(super) fn render_with(
    pieces: &[Piece<'_>],
    context: &Value,
    escaping: bool,
    overrides: &HashMap<String, String>,
) -> Result<String, String> {
    let mut out = String::new();
    let mut index = 0usize;
    while index < pieces.len() {
        index = super::template_step::step(pieces, index, context, escaping, overrides, &mut out)?;
    }
    Ok(out)
}

pub(super) use super::template_delimit::matching_end;
pub(super) use super::template_subject::{condition, iterable};
