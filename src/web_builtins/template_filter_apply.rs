//! Filter application.
//!
//! Filters operate on the resolved value, and one of them — `safe` — changes how the
//! result is emitted rather than what it contains. That is why application returns
//! both a value and an escaping decision instead of just text.

use super::template_filter::Filter;
use crate::value::Value;

/// A filtered value plus whether it must still be escaped.
pub(super) struct Filtered {
    /// The value after every filter ran.
    pub value: Value,
    /// False once `safe` has been applied.
    pub escape: bool,
}

/// Apply `filters` left to right.
///
/// # Arguments
///
/// * `value` — Resolved value, or `None` when the key was absent.
/// * `filters` — Pipeline in source order.
/// * `escaping` — Whether escaping is on for this render.
///
/// # Errors
///
/// Returns an error for an unknown filter, a malformed argument, or a missing key
/// that no `default` filter supplied.
pub(super) fn apply(
    value: Option<Value>,
    filters: &[Filter<'_>],
    escaping: bool,
) -> Result<Filtered, String> {
    let mut current = value;
    let mut escape = escaping;
    for filter in filters {
        match filter.name {
            // `safe` marks content as intentionally raw; it is the only filter that
            // affects emission rather than the value.
            "safe" => escape = false,
            "default" => {
                // The argument is validated even when the value is present, so a
                // malformed `default()` is caught on every render rather than only
                // on the rows where the key happens to be missing.
                let fallback = super::template_filter_arg::parse(filter.argument)?;
                if current.is_none() || matches!(current, Some(Value::Nil)) {
                    current = Some(fallback);
                }
            }
            other => current = Some(super::template_filter_fn::call(other, current, filter)?),
        }
    }
    let value = current.ok_or("template: value is missing and no `default` was given")?;
    Ok(Filtered { value, escape })
}
