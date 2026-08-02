//! Template rendering entry point.
//!
//! Scans the template into pieces, then evaluates them. Scanning first means the
//! block structure is validated before any value is looked up, so an unbalanced
//! `{% if %}` is reported as such rather than as a missing key.

use super::template_block;
use super::template_scan::scan;
use crate::value::Value;

/// Render `template` against `context`.
///
/// # Arguments
///
/// * `template` — Source text with `{{ name }}`, `{{{ raw }}}`, and `{% ... %}`.
/// * `context` — Map supplying values.
/// * `escaping` — When true, `{{ }}` output is HTML-escaped.
///
/// # Returns
///
/// The rendered text.
///
/// # Errors
///
/// Returns an error for an unclosed delimiter, an unbalanced block, an unsupported
/// tag, or a failed lookup.
pub(super) fn render(template: &str, context: &Value, escaping: bool) -> Result<String, String> {
    let pieces = scan(template)?;
    template_block::render(&pieces, context, escaping)
}
