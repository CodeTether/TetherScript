//! Condition evaluation with `and`/`or`/`is defined` and comparisons.

use super::template_filter_arg::literal_of;
use super::template_subject::{ordered, truthy};
use crate::value::Value;

const OPS: [&str; 6] = ["==", "!=", ">=", "<=", ">", "<"];

pub(super) fn evaluate(context: &Value, expression: &str) -> Result<bool, String> {
    let expr = expression.trim();
    if let Some((l, r)) = expr.find(" or ").map(|p| (&expr[..p], &expr[p + 4..])) {
        return Ok(evaluate(context, l)? || evaluate(context, r)?);
    }
    if let Some((l, r)) = expr.find(" and ").map(|p| (&expr[..p], &expr[p + 5..])) {
        return Ok(evaluate(context, l)? && evaluate(context, r)?);
    }
    if let Some(s) = expr.strip_suffix(" is defined") {
        return Ok(is_defined(context, s.trim()));
    }
    if let Some(s) = expr.strip_suffix(" is not defined") {
        return Ok(!is_defined(context, s.trim()));
    }
    for op in OPS {
        if let Some((l, r)) = expr.split_once(op) {
            let (lv, rv) = (operand(context, l.trim()), operand(context, r.trim()));
            return Ok(match op {
                "==" => equal(&lv, &rv),
                "!=" => !equal(&lv, &rv),
                _ => ordered(&lv, op, &rv).unwrap_or(false),
            });
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

fn is_defined(context: &Value, key: &str) -> bool {
    !matches!(
        super::template_context::lookup_value(context, key),
        Ok(Value::Nil) | Err(_)
    )
}
