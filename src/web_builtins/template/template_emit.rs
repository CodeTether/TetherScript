//! Filter application and expression emission.
//!
//! One concern: turning `{{ value | filter | filter }}` into output text. `safe` is the reason
//! application returns an escaping decision alongside a value — it changes how the result is emitted
//! rather than what it contains.

use super::template_block::Render;
use super::template_context::{lookup_value, render_scalar};
use super::template_escape::escape;
use super::template_filter::{split, Filter};
use crate::value::Value;

/// Render one hole body, honouring its filters.
///
/// # Errors
///
/// Returns an error for a malformed pipeline, an unknown filter, or a missing key that no `default`
/// supplied and no lenient mode tolerated.
pub(super) fn emit(body: &str, context: &Value, state: &Render<'_>) -> Result<String, String> {
    let (key, filters) = split(body)?;

    let resolved = match lookup_value(context, key) {
        Ok(value) => Some(value),
        Err(error) => {
            // A `default` filter supplies the value, and lenient mode tolerates the absence
            // outright — which is what lets a view tree written against Tera's own lenient default
            // render without every unmapped key taking the page down.
            if !state.lenient && !filters.iter().any(|filter| filter.name == "default") {
                return Err(error);
            }
            // Nil rather than None, so `apply` has something to work with and `render_scalar` turns
            // it into the empty string — matching what Tera emits for a missing variable.
            Some(Value::Nil)
        }
    };

    let (value, escaping) = apply(resolved, &filters, state.escaping)?;
    let text = render_scalar(&value, key)?;
    Ok(if escaping { escape(&text) } else { text })
}

/// Apply `filters` left to right, returning the value and whether to escape it.
///
/// # Errors
///
/// Returns an error for an unknown filter, a malformed argument, or a missing value that no
/// `default` supplied.
pub(super) fn apply(
    value: Option<Value>,
    filters: &[Filter<'_>],
    escaping: bool,
) -> Result<(Value, bool), String> {
    let mut current = value;
    let mut escape = escaping;
    for filter in filters {
        match filter.name {
            "safe" => escape = false,
            "default" => current = super::template_emit_default::apply(current, filter)?,
            other => current = Some(super::template_filter_fn::call(other, current, filter)?),
        }
    }
    let value = current.ok_or("template: value is missing and no `default` was given")?;
    Ok((value, escape))
}
