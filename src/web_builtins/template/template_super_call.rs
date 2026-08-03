//! Rendering the parent block body for `{{ super() }}`.
//!
//! Split from the hole dispatcher so that file stays within the line budget.

use super::template_block::{render_with, Render};
use super::template_scan::scan;
use crate::value::Value;

/// Render the parent block's body in place of `{{ super() }}`.
///
/// The body is re-scanned rather than reused as pieces, because the parent block may itself contain
/// `if`, `for`, or nested blocks that must evaluate against the *child's* context — which is what
/// makes `super()` useful rather than a verbatim paste.
///
/// # Errors
///
/// Returns an error when `super()` appears outside an overriding block, where there is no parent to
/// render. Silently emitting nothing would hide a template mistake whose only symptom is missing
/// content — a dropped stylesheet link, for instance.
pub(super) fn render(context: &Value, state: &Render<'_>) -> Result<String, String> {
    let Some(parent) = super::template_super::parent_of(context) else {
        return Err(
            "template: `super()` is only valid inside a block that overrides a parent's"
                .to_string()
                + " block",
        );
    };
    render_with(&scan(&parent)?, context, state)
}
