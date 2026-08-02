//! Condition and loop-subject resolution.

use super::template_context::lookup_value;
use super::template_truth::truthy;
use crate::value::Value;

/// Resolve the loop subject, which must be a list.
///
/// # Errors
///
/// Returns an error naming the key when the value is not a list: iterating a scalar
/// once, silently, would hide the mistake.
pub(super) fn iterable(context: &Value, key: &str) -> Result<Vec<Value>, String> {
    match lookup_value(context, key)? {
        Value::List(items) => Ok(items.borrow().clone()),
        other => Err(format!(
            "template: `{key}` must be a list to loop over, got {}",
            other.type_name()
        )),
    }
}

/// Whether a condition key is satisfied.
///
/// A missing key is false rather than an error, because `{% if user %}` is the
/// idiomatic way to ask whether an optional value is present.
///
/// # Errors
///
/// Returns an error only when the context itself is malformed.
pub(super) fn condition(context: &Value, key: &str) -> Result<bool, String> {
    match lookup_value(context, key) {
        Ok(value) => Ok(truthy(&value)),
        Err(_) => Ok(false),
    }
}
