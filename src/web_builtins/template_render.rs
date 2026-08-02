//! Template rendering entry point.
//!
//! Resolves `{% extends %}` first, then evaluates the resulting root template with
//! the child's block overrides applied. Scanning before evaluation means block
//! structure is validated before any value is looked up, so an unbalanced
//! `{% if %}` is reported as such rather than as a missing key.

use super::template_block::render_with;
use super::template_inherit::resolve;
use super::template_scan::scan;
use crate::value::Value;

/// Render `template` against `context`, with no inheritance.
///
/// # Arguments
///
/// * `template` — Source text with `{{ name }}`, `{{{ raw }}}`, and `{% ... %}`.
/// * `context` — Map supplying values.
/// * `escaping` — When true, `{{ }}` output is HTML-escaped.
///
/// # Errors
///
/// Returns an error for an unclosed delimiter, an unbalanced block, an unsupported
/// tag, or a failed lookup.
pub(super) fn render(template: &str, context: &Value, escaping: bool) -> Result<String, String> {
    render_inherited(template, context, &Value::Nil, escaping)
}

/// Render `template`, resolving `{% extends %}` against `templates`.
///
/// # Arguments
///
/// * `templates` — Map of template name to source, for ancestors. `Value::Nil` when
///   inheritance is not in use.
///
/// # Errors
///
/// Additionally returns an error for a missing parent or a cyclic chain.
pub(super) fn render_inherited(
    template: &str,
    context: &Value,
    templates: &Value,
    escaping: bool,
) -> Result<String, String> {
    let resolved = resolve(template, templates)?;
    let pieces = scan(&resolved.root)?;
    render_with(&pieces, context, escaping, &resolved.overrides)
}
