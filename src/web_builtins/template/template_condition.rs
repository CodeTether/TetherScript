//! Condition expression evaluation.
//!
//! A condition is either a bare key tested for truthiness, or a comparison. Comparisons
//! are the common case in real views — 443 of them in the reference — and treating one as
//! a bare key would silently take the wrong branch rather than fail, which is why this
//! parses the operators rather than ignoring them.

use super::template_filter_arg::literal_of;
use super::template_subject::{ordered, truthy};
use crate::value::Value;

/// Operators recognized in a condition, longest first so `==` is not read as `=`.
const OPERATORS: [&str; 6] = ["==", "!=", ">=", "<=", ">", "<"];

/// Evaluate a condition expression.
///
/// # Arguments
///
/// * `context` — Root context map.
/// * `expression` — Everything after `if` or `elif`.
///
/// # Returns
///
/// Whether the branch should be taken.
///
/// # Errors
///
/// Returns an error when a comparison operand cannot be resolved. A bare missing key is
/// false rather than an error, since `{% if user %}` is how a view tests presence.
pub(super) fn evaluate(context: &Value, expression: &str) -> Result<bool, String> {
    let expression = expression.trim();
    for operator in OPERATORS {
        if let Some((left, right)) = expression.split_once(operator) {
            return compare(context, left.trim(), operator, right.trim());
        }
    }
    // A bare condition may still carry filters, as in `{% if items | length %}`.
    Ok(truthy(&operand(context, expression)))
}

/// Resolve both sides and apply `operator`.
fn compare(context: &Value, left: &str, operator: &str, right: &str) -> Result<bool, String> {
    let left = operand(context, left);
    let right = operand(context, right);
    Ok(match operator {
        "==" => equal(&left, &right),
        "!=" => !equal(&left, &right),
        _ => ordered(&left, operator, &right)?,
    })
}

/// Resolve a side as a context key, falling back to a literal.
///
/// Filters apply here too. `{% if testimonials | length > 0 %}` is the shape a real stored page
/// uses, and treating `testimonials | length` as one long key name found nothing — so the
/// comparison saw nil and failed, taking the whole page down rather than one condition.
///
/// A missing key yields `nil`, which makes `{% if absent == "x" %}` false rather than an
/// error — the same tolerance a bare key gets.
fn operand(context: &Value, text: &str) -> Value {
    if text.starts_with('"') || text.starts_with('\'') || text.starts_with(|c: char| c.is_numeric())
    {
        return literal_of(text);
    }
    match text {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        // A malformed pipeline or unknown filter yields nil rather than an error, keeping a
        // condition's existing tolerance: an untaken branch must not be able to fail a render.
        _ => super::template_emit_default::value_of(text, context).unwrap_or(Value::Nil),
    }
}

/// Structural equality across the scalar types a template can compare.
fn equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Str(a), Value::Str(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Int(a), Value::Float(b)) | (Value::Float(b), Value::Int(a)) => (*a as f64) == *b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        _ => false,
    }
}
