//! The `default` filter, and resolving an expression to a value rather than to text.
//!
//! Split from [`super::template_emit`] so each file stays within the line budget.

use super::template_filter::{split, Filter};
use crate::value::Value;

/// Apply `default(value=..)` to the value so far.
///
/// The argument is parsed even when the value is present, so a malformed `default()` is caught on
/// every render rather than only on the rows where the key happens to be missing.
///
/// # Errors
///
/// Returns an error for a missing or malformed argument.
pub(super) fn apply(current: Option<Value>, filter: &Filter<'_>) -> Result<Option<Value>, String> {
    let supplied = super::template_filter_arg::parse(filter.argument)?;
    match current {
        None | Some(Value::Nil) => Ok(Some(supplied)),
        present => Ok(present),
    }
}

/// Resolve an expression to a value, applying its filters but not rendering it.
///
/// A condition needs the value rather than the text: `{% if testimonials | length > 0 %}` has to
/// compare a number, and rendering to a string first would make it compare `"0"` against `0`.
/// Without this, a filtered operand fell through to a bare key lookup, found nothing, and the
/// comparison failed with a type error — which took a whole stored page down with it.
///
/// # Arguments
///
/// * `body` — The expression, possibly containing `|` filters.
/// * `context` — Root context map.
///
/// # Errors
///
/// Returns an error for a malformed pipeline or an unknown filter. A missing key is `nil` rather than
/// an error, matching the tolerance a bare key already gets in a condition.
pub(super) fn value_of(body: &str, context: &Value) -> Result<Value, String> {
    let (key, filters) = split(body)?;
    let resolved = super::template_context::lookup_value(context, key).ok();
    match filters.is_empty() {
        true => Ok(resolved.unwrap_or(Value::Nil)),
        // Escaping is irrelevant here: nothing is emitted, so `safe` is accepted and ignored.
        false => super::template_emit::apply(resolved, &filters, false).map(|(value, _)| value),
    }
}
