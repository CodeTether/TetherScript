//! `{% include %}` rendering.
//!
//! An include splices another template's output in place, sharing the current
//! context. Unlike `extends`, it is not structural: the included template does not
//! participate in block inheritance, matching Tera and keeping the block map from
//! being mutated mid-render.

use super::template_block::render_with;
use super::template_block::Render;
use super::template_extends::{source_of, unquote};
use super::template_scan::scan;
use crate::value::Value;

/// Depth limit for nested includes.
///
/// A template that includes itself would otherwise recurse until the stack overflowed,
/// which aborts the process instead of reporting an error.
const MAX_DEPTH: usize = 16;

/// Render an `include` tag into `out`.
///
/// # Errors
///
/// Returns an error for a malformed name, a missing template without
/// `ignore missing`, or nesting deeper than [`MAX_DEPTH`].
pub(super) fn run(
    body: &str,
    context: &Value,
    state: &Render<'_>,
    out: &mut String,
) -> Result<(), String> {
    if state.depth >= MAX_DEPTH {
        return Err(format!(
            "template: include nested deeper than {MAX_DEPTH}; a partial may include itself"
        ));
    }
    let name = unquote(body)?;
    let source = match source_of(state.templates, name) {
        Ok(source) => source,
        // `ignore missing` makes an absent partial render as nothing, which the
        // reference relies on for optional sections.
        Err(_) if body.contains("ignore missing") => return Ok(()),
        Err(error) => return Err(error),
    };
    let nested = state.nested(state.escaping, true);
    out.push_str(&render_with(&scan(&source)?, context, &nested)?);
    Ok(())
}
