//! Length and numeric-coercion filters.
//!
//! Split from [`super::template_filter_fn`] so each file stays within the line budget.

use std::rc::Rc;

use crate::value::Value;

/// Length of a string, list, or map.
///
/// Counts characters rather than bytes, so a multi-byte name reports the length a template
/// author would expect.
///
/// # Errors
///
/// Returns an error naming the type for anything without a length.
pub(super) fn length_of(value: &Value) -> Result<Value, String> {
    let length = match value {
        Value::Str(text) => text.chars().count(),
        Value::List(items) => items.borrow().len(),
        Value::Map(entries) => entries.borrow().len(),
        other => {
            return Err(format!(
                "template: `length` needs a str, list, or map, got {}",
                other.type_name()
            ))
        }
    };
    Ok(Value::Int(length as i64))
}

/// Coerce a value with the `int`, `float`, or `str` filter.
///
/// # Errors
///
/// Returns an error naming both the filter and the value when the conversion cannot be
/// made, rather than substituting a zero that would silently corrupt a page.
pub(super) fn coerce(name: &str, value: &Value) -> Result<Value, String> {
    match (name, value) {
        ("str", _) => Ok(Value::Str(Rc::new(text_of(value)))),
        ("int", Value::Int(_)) | ("float", Value::Float(_)) => Ok(value.clone()),
        ("int", Value::Float(number)) => Ok(Value::Int(*number as i64)),
        ("float", Value::Int(number)) => Ok(Value::Float(*number as f64)),
        ("int", Value::Str(text)) => text
            .trim()
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| format!("template: `int` cannot parse `{text}`")),
        ("float", Value::Str(text)) => text
            .trim()
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| format!("template: `float` cannot parse `{text}`")),
        _ => Err(format!(
            "template: `{name}` cannot convert {}",
            value.type_name()
        )),
    }
}

/// Display text for a value, used by the `str` filter.
fn text_of(value: &Value) -> String {
    match value {
        Value::Str(text) => (**text).clone(),
        Value::Nil => String::new(),
        other => format!("{other}"),
    }
}
