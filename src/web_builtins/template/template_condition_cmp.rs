//! Atomic condition comparison: `==`, `!=`, `>`, `<`, etc.

use super::template_filter_arg::literal_of;
use super::template_subject::{ordered, truthy};
use crate::value::Value;

const OPS: [&str; 6] = ["==", "!=", ">=", "<=", ">", "<"];

/// Evaluate an atomic condition (no `and`/`or`).
pub(super) fn evaluate(context: &Value, expr: &str) -> Result<bool, String> {
    for op in OPS {
        if let Some((l, r)) = expr.split_once(op) {
            let (lv, rv) = (operand(context, l.trim()), operand(context, r.trim()));
            return match op {
                "==" => Ok(equal(&lv, &rv)),
                "!=" => Ok(!equal(&lv, &rv)),
                // Propagate: `ordered` names the offending types, and swallowing
                // that into `false` silently takes the untaken branch instead.
                _ => ordered(&lv, op, &rv),
            };
        }
    }
    Ok(truthy(&operand(context, expr)))
}

fn operand(context: &Value, text: &str) -> Value {
    if text.starts_with('"') || text.starts_with('\'') || text.starts_with(|c: char| c.is_numeric())
    {
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
