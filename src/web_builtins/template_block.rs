//! Statement evaluation for `{% if %}` and `{% for %}`.
//!
//! Together these are the overwhelming majority of tags in the reference views, so
//! they are the subset implemented first. Evaluation walks the piece list with an
//! explicit index rather than recursing over slices, which keeps nesting depth
//! independent of the Rust stack.

use super::template_scan::Piece;
use crate::value::Value;

/// Render `pieces` against `context`.
///
/// # Arguments
///
/// * `pieces` — Scanned template.
/// * `context` — Root context map.
/// * `escaping` — Whether `{{ }}` output is HTML-escaped.
///
/// # Errors
///
/// Returns an error for an unknown key, an unbalanced block, or an unsupported tag.
pub(super) fn render(
    pieces: &[Piece<'_>],
    context: &Value,
    escaping: bool,
) -> Result<String, String> {
    let mut out = String::new();
    let mut index = 0usize;
    while index < pieces.len() {
        index = super::template_step::step(pieces, index, context, escaping, &mut out)?;
    }
    Ok(out)
}

pub(super) use super::template_bounds::matching_end;
pub(super) use super::template_subject::{condition, iterable};
