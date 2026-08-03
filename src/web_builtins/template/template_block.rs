//! Render state and the evaluation loop.
//!
//! State is bundled into one [`Render`] value rather than passed as four parameters:
//! `include` needs the template map and a depth counter, and threading those through
//! every signature individually would push several files over the line budget.
//!
//! `nested` returns a fresh value rather than mutating, so a nested render cannot alter
//! its parent's state — an include one level down must not leave escaping disabled for
//! whatever follows it.

use std::collections::HashMap;

use super::template_scan::Piece;
use crate::value::Value;

/// Everything an evaluation step needs besides the pieces and the context.
pub(super) struct Render<'a> {
    /// Whether `{{ }}` output is HTML-escaped.
    pub escaping: bool,
    /// Block bodies supplied by a more-derived template.
    pub overrides: &'a HashMap<String, String>,
    /// Map of template name to source, for `extends` and `include`.
    pub templates: &'a Value,
    /// Current include nesting depth, for cycle detection.
    pub depth: usize,
}

impl<'a> Render<'a> {
    /// State for a render whose templates and overrides the caller owns.
    pub(super) fn new(
        escaping: bool,
        overrides: &'a HashMap<String, String>,
        templates: &'a Value,
    ) -> Self {
        Self {
            escaping,
            overrides,
            templates,
            depth: 0,
        }
    }

    /// The same state, optionally one include deeper and with escaping overridden.
    pub(super) fn nested(&self, escaping: bool, deeper: bool) -> Render<'a> {
        Render {
            escaping,
            overrides: self.overrides,
            templates: self.templates,
            depth: self.depth + usize::from(deeper),
        }
    }
}

/// Render `pieces` against `context` with the given render state.
///
/// Walks the piece list with an explicit index rather than recursing over slices, which
/// keeps nesting depth independent of the Rust stack.
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

pub(super) use super::template_delimit::matching_end;
pub(super) use super::template_subject::iterable;
