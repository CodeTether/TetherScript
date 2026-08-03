//! The evaluation loop.
//!
//! Split from [`super::template_block`] so the render state and the loop that consumes it stay
//! separable, and each file stays within the line budget.

use super::template_block::Render;
use super::template_scan::Piece;
use crate::value::Value;

/// Render `pieces` against `context` with the given render state.
///
/// Walks the piece list with an explicit index rather than recursing over slices, which keeps
/// nesting depth independent of the Rust stack — a template can nest far deeper than a recursive
/// walk would survive.
///
/// # Arguments
///
/// * `pieces` — Scanned template pieces, in source order.
/// * `context` — Map supplying values.
/// * `state` — Escaping, block overrides, the template map, and the include depth.
///
/// # Returns
///
/// The rendered text.
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
        index = super::template_step::step(pieces, index, context, state, &mut out)?;
    }
    Ok(out)
}
