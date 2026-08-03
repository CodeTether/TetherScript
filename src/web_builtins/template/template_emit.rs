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
