//! Truthiness for `{% if %}` conditions.
//!
//! Follows Tera/Jinja rather than tetherscript's own rules: a template asking
//! `{% if items %}` means "is there anything to show", so an empty list and an
//! empty string are both false. Requiring `items.len() > 0` in every view would
//! make ported templates diverge from their originals.

use crate::value::Value;

/// Whether `value` should take the `if` branch.
///
/// # Arguments
///
/// * `value` — Resolved condition value.
///
/// # Returns
///
/// False for `nil`, `false`, zero, an empty string, and an empty list or map.
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
