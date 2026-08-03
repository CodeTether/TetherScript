//! The single hole-evaluation hook macro support adds.
//!
//! Exists so the integrator's change to [`super::template_step`] is a one-for-one
//! substitution rather than an added branch: that file is already near the line budget, and
//! a `{{ }}` hole is now either a macro call or a value expression.
//!
//! Only the double-brace form is routed here. `{{{ raw }}}` stays a value expression,
//! because a macro's own output already honours escaping hole by hole and wrapping a call
//! in the raw form would blanket-disable it.

use super::template_block::Render;
use super::template_emit::emit;
use super::template_macro_call::{call, is_call};
use super::template_scan::Piece;
use crate::value::Value;

/// Render one `{{ }}` hole, dispatching a macro call or a value expression.
///
/// # Arguments
///
/// * `pieces` — Pieces of the template being rendered, so a same-template call resolves.
/// * `name` — Trimmed hole body.
/// * `context` — Current context.
/// * `state` — Current render state.
///
/// # Returns
///
/// The hole's rendered text.
///
/// # Errors
///
/// Propagates whichever path was taken: a macro-call failure, or a lookup or filter
/// failure from [`emit`].
///
/// # Examples
///
/// ```text
/// hole(pieces, "ui::badge(kind=\"new\")", ctx, state)  →  the macro's output
/// hole(pieces, "title | upper", ctx, state)            →  the value's output
/// ```
pub(super) fn hole(
    pieces: &[Piece<'_>],
    name: &str,
    context: &Value,
    state: &Render<'_>,
) -> Result<String, String> {
    // `super()` looks like a macro call but is not one: it re-renders the parent block's body,
    // which the enclosing `{% block %}` bound into the context before this override ran.
    if name.trim() == "super()" {
        return super::template_super_call::render(context, state);
    }
    if is_call(name) {
        return call(pieces, name, context, state);
    }
    emit(name, context, state)
}
