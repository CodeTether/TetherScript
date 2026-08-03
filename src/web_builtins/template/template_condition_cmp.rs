//! Atomic condition comparison: `==`, `!=`, `>`, `<`, etc.

use super::template_filter_arg::literal_of;
use super::template_subject::{ordered, truthy};
use crate::value::Value;

const OPERATORS: [&str; 6] = ["==", "!=", ">=", "<=", ">", "<"];

/// Evaluate an atomic condition (no `and`/`or`).
pub(super) fn evaluate(context: &Value, expr: &str) -> Result<bool, String> {
    for op in OPERATORS {
        if let Some((l, r)) = expr.split_once(op) {
            return Ok(compare(context, l.trim(), op, r.trim()));
        }
    }
    Ok(truthy(&operand(context, expr)))
}

fn compare(context: &Value, left: &str, op: &str, right: &str) -> bool {
    let l = operand(context, left);
    let r = operand(context, right);
    match op {
        "==" => equal(&l, &r),
        "!=" => !equal(&l, &r),
        _ => ordered(&l, op, &r).unwrap_or(false),
    }
}

fn operand(context: &Value, text: &str) -> Value {
    if text.starts_with('"') || text.starts_with('\'') || text.starts_with(|c: char| c.is_numeric()) {
        return literal_of(text);
    }
    match text {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => super::template_emit_default::value_of(text, context).unwrap_or(Value::Nil),
    }
}

fn equal(l: &Value, r: &Value) -> bool {
    match (l, r) {
        (Value::Str(a), Value::Str(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        _ => false,
    }
}
