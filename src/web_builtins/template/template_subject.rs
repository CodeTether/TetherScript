//! Value semantics for conditions and loops.
//!
//! Deciding what a template value *means* — is it iterable, is it true, how does it order
//! against another — is one concern, so all three live here.

use super::template_context::lookup_value;
use crate::value::Value;

/// Resolve the loop subject, which must be a list.
///
/// # Errors
///
/// Returns an error naming the key when the value is not a list: iterating a scalar once,
/// silently, would hide the mistake.
pub(super) fn iterable(context: &Value, key: &str) -> Result<Vec<Value>, String> {
    match lookup_value(context, key)? {
        Value::List(items) => Ok(items.borrow().clone()),
        other => Err(format!(
            "template: `{key}` must be a list to loop over, got {}",
            other.type_name()
        )),
    }
}

/// Whether `value` should take the `if` branch.
///
/// Follows Tera/Jinja rather than tetherscript's own rules: a template asking
/// `{% if items %}` means "is there anything to show", so an empty list and an empty
/// string are both false. Requiring `items.len() > 0` in every view would make ported
/// templates diverge from their originals.
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

/// Apply `>`, `<`, `>=`, or `<=` to two numeric operands.
///
/// Only numbers are ordered: comparing strings with `<` is almost always a mistake in a
/// template, and comparing them lexicographically would hide it.
///
/// # Errors
///
/// Returns an error naming the types when either side is not numeric.
pub(super) fn ordered(left: &Value, operator: &str, right: &Value) -> Result<bool, String> {
    let (Some(a), Some(b)) = (number(left), number(right)) else {
        return Err(format!(
            "template: `{operator}` needs numbers, got {} and {}",
            left.type_name(),
            right.type_name()
        ));
    };
    Ok(match operator {
        ">" => a > b,
        "<" => a < b,
        ">=" => a >= b,
        "<=" => a <= b,
        // Unreachable: the caller only routes ordering operators here.
        _ => return Err(format!("template: unknown operator `{operator}`")),
    })
}

/// Numeric value of an int or float, if it is one.
fn number(value: &Value) -> Option<f64> {
    match value {
        Value::Int(number) => Some(*number as f64),
        Value::Float(number) => Some(*number),
        _ => None,
    }
}
