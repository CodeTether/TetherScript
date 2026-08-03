//! Renderable piece assembly for a macro body.
//!
//! A macro body is rendered as its own piece list, which means the definitions of the
//! template it came from are no longer in view. Recursion — `{% macro tree(node) %}` calling
//! `{{ tree(node=child) }}` — and sibling calls both depend on them being visible, so every
//! definition from the defining template is re-emitted ahead of the body.
//!
//! Re-emitting definitions is cheap because a definition renders as nothing: it is skipped
//! by [`super::template_macro::run`]. The alternative, threading a macro table through
//! [`super::template_block::Render`], would mean editing a file this change does not own.

use super::template_macro::{Macro, Macros};
use super::template_scan::Piece;

/// Build the piece list to render for one macro call.
///
/// # Arguments
///
/// * `macros` — Every macro the defining template declares.
/// * `def` — The macro being called, whose body ends the list.
///
/// # Returns
///
/// Each definition restated as `macro`-header/body/`endmacro`, followed by the called
/// macro's body. The definitions emit nothing; they exist so a nested call resolves.
///
/// # Errors
///
/// None; assembly is pure list construction. Failures surface when the result is rendered.
///
/// # Examples
///
/// ```text
/// definitions for \{% macro tree(n) %\}…\{% endmacro %\} plus the body of `tree`
/// ```
pub(super) fn assemble<'a>(macros: &Macros<'a>, def: &Macro<'a>) -> Vec<Piece<'a>> {
    let mut pieces = Vec::new();
    for entry in macros.values() {
        pieces.push(Piece::Tag(entry.header));
        pieces.extend(entry.body.iter().cloned());
        // The bare end form, since the trailing name is only an annotation.
        pieces.push(Piece::Tag("endmacro"));
    }
    pieces.extend(def.body.iter().cloned());
    pieces
}
