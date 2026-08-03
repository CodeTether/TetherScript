//! Collection and rounding filters.
//!
//! Split by concern from the string and coercion filters so each file stays within the
//! line budget.

use crate::value::Value;

/// Apply `first`, `last`, `round`, or `truncate`.
///
/// # Errors
///
/// Returns an error naming the filter and the value's type when it does not apply.
pub(super) fn call(name: &str, value: &Value, argument: &str) -> Result<Value, String> {
    match name {
        "first" => end_of(value, true),
        "last" => end_of(value, false),
        "round" => round(value),
        "truncate" => super::template_filter_text::truncate(value, argument),
        _ => Err(format!("template: unknown filter `{name}`")),
    }
}

/// First or last element of a list.
///
/// An empty list yields `nil` rather than an error: `{{ items | first }}` on an empty list
/// is a legitimate blank, and a view guards it with `{% if items %}` when it matters.
fn end_of(value: &Value, first: bool) -> Result<Value, String> {
    let Value::List(items) = value else {
        return Err(format!(
            "template: `first`/`last` need a list, got {}",
            value.type_name()
        ));
    };
    let items = items.borrow();
    Ok(if first {
        items.first().cloned().unwrap_or(Value::Nil)
    } else {
        items.last().cloned().unwrap_or(Value::Nil)
    })
}

/// Round a float to the nearest integer, leaving ints alone.
fn round(value: &Value) -> Result<Value, String> {
    match value {
        Value::Float(number) => Ok(Value::Int(number.round() as i64)),
        Value::Int(_) => Ok(value.clone()),
        other => Err(format!(
            "template: `round` needs a number, got {}",
            other.type_name()
        )),
    }
}
