//! Length-bound rules for `validate_fields`.
//!
//! Separated from the rule dispatcher because these are the only rules that read
//! a bound argument, so they are the only ones that can be misconfigured.

use crate::value::Value;

/// Compare a character count against an inclusive bound.
///
/// # Arguments
///
/// * `field` — Field name, used in the returned message.
/// * `text` — Rendered field value.
/// * `spec` — The bound; must be an int.
/// * `minimum` — True for `min_len`, false for `max_len`.
///
/// # Returns
///
/// `Ok(None)` when the length is acceptable, `Ok(Some(message))` when it is not.
///
/// # Errors
///
/// Returns an error naming the field when the bound is not an int, since a
/// misconfigured rule must not silently pass every value.
pub(super) fn length(
    field: &str,
    text: &str,
    spec: &Value,
    minimum: bool,
) -> Result<Option<String>, String> {
    let Value::Int(bound) = spec else {
        return Err(format!(
            "validate_fields: {field} length bound must be int, got {}",
            spec.type_name()
        ));
    };
    // Count characters, not bytes: a byte length would reject short strings that
    // merely contain non-ASCII, so "café" would fail a max_len of 4.
    let len = text.chars().count() as i64;
    let failed = if minimum { len < *bound } else { len > *bound };
    Ok(failed.then(|| {
        if minimum {
            format!("{field} must be at least {bound} characters")
        } else {
            format!("{field} must be at most {bound} characters")
        }
    }))
}
