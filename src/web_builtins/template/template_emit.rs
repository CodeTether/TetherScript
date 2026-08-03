//! Filter application and expression emission.
//!
//! One concern: turning `{{ value | filter | filter }}` into output text. `safe` is the
//! reason application returns an escaping decision alongside a value — it changes how
//! the result is emitted rather than what it contains.

use super::template_block::Render;
use super::template_context::{lookup_value, render_scalar};
use super::template_escape::escape;
use super::template_filter::{split, Filter};
use crate::value::Value;

/// Render one hole body, honouring its filters.
///
/// # Errors
///
/// Returns an error for a malformed pipeline, an unknown filter, or a missing key that
/// no `default` supplied.
pub(super) fn emit(body: &str, context: &Value, state: &Render<'_>) -> Result<String, String> {
    let (key, filters) = split(body)?;

    // A missing key is only tolerable when a `default` filter follows, so absence is
    // carried as None rather than being an immediate error.
    let resolved = match lookup_value(context, key) {
        Ok(value) => Some(value),
        Err(error) => {
            if !filters.iter().any(|filter| filter.name == "default") {
                return Err(error);
            }
            None
        }
    };

    let (value, escaping) = apply(resolved, &filters, state.escaping)?;
    let text = render_scalar(&value, key)?;
    Ok(if escaping { escape(&text) } else { text })
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
/// Returns an error for a malformed pipeline or an unknown filter. A missing key is `nil` rather
/// than an error, matching the tolerance a bare key already gets in a condition.
pub(super) fn value_of(body: &str, context: &Value) -> Result<Value, String> {
    let (key, filters) = split(body)?;
    let resolved = lookup_value(context, key).ok();
    if filters.is_empty() {
        return Ok(resolved.unwrap_or(Value::Nil));
    }
    // Escaping is irrelevant here: nothing is emitted, so `safe` is accepted and ignored.
    let (value, _) = apply(resolved, &filters, false)?;
    Ok(value)
}

/// Apply `filters` left to right, returning the value and whether to escape it.
///
/// # Errors
///
/// Returns an error for an unknown filter, a malformed argument, or a missing value that
/// no `default` supplied.
fn apply(
    value: Option<Value>,
    filters: &[Filter<'_>],
    escaping: bool,
) -> Result<(Value, bool), String> {
    let mut current = value;
    let mut escape = escaping;
    for filter in filters {
        match filter.name {
            "safe" => escape = false,
            "default" => {
                // Validated even when the value is present, so a malformed `default()`
                // is caught on every render rather than only on missing rows.
                let supplied = super::template_filter_arg::parse(filter.argument)?;
                if current.is_none() || matches!(current, Some(Value::Nil)) {
                    current = Some(supplied);
                }
            }
            other => current = Some(super::template_filter_fn::call(other, current, filter)?),
        }
    }
    let value = current.ok_or("template: value is missing and no `default` was given")?;
    Ok((value, escape))
}
