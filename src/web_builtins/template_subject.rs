//! Condition and loop-subject resolution.
//!
//! Truthiness lives here too, since deciding whether a condition holds and deciding
//! what a condition *means* are the same concern.

use super::template_context::lookup_value;
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

/// Whether `value` should take the `if` branch.
///
/// Follows Tera/Jinja rather than tetherscript's own rules: a template asking
/// `{% if items %}` means "is there anything to show", so an empty list and an empty
/// string are both false. Requiring `items.len() > 0` in every view would make
/// ported templates diverge from their originals.
pub(super) fn truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(flag) => *flag,
        Value::Int(number) => *number != 0,
        Value::Float(number) => *number != 0.0,
        Value::Str(text) => !text.is_empty(),
        Value::List(items) => !items.borrow().is_empty(),
        Value::Map(entries) => !entries.borrow().is_empty(),
        // Anything else present is something, so it shows.
        _ => true,
    }
}
