//! Expression emission for `{{ ... }}` holes.
//!
//! Handles the filter pipeline, so `{{ x | json | safe }}` resolves `x`, encodes it,
//! and suppresses escaping — the exact combination the reference views use to embed
//! data in a `<script>` block.

use super::template_context::{lookup_value, render_scalar};
use super::template_escape::escape;
use super::template_filter::split;
use super::template_filter_apply::apply;
use crate::value::Value;

/// Render one hole body, honouring its filters.
///
/// # Arguments
///
/// * `body` — Text inside the braces, possibly containing `|` filters.
/// * `context` — Root context map.
/// * `escaping` — Whether escaping is on, before any `safe` filter.
///
/// # Errors
///
/// Returns an error for a malformed pipeline, an unknown filter, or a missing key
/// that no `default` supplied.
pub(super) fn emit(body: &str, context: &Value, escaping: bool) -> Result<String, String> {
    let (key, filters) = split(body)?;

    // A missing key is only tolerable when a `default` filter follows, so absence is
    // carried as None rather than being an immediate error.
    let resolved = match lookup_value(context, key) {
        Ok(value) => Some(value),
        Err(error) if filters.iter().any(|f| f.name == "default") => {
            let _ = error;
            None
        }
        Err(error) => return Err(error),
    };

    let filtered = apply(resolved, &filters, escaping)?;
    let text = render_scalar(&filtered.value, key)?;
    Ok(if filtered.escape { escape(&text) } else { text })
}
