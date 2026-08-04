//! Condition evaluation with `and`/`or`/`is defined` and comparisons.
//!
//! Split: logical combinators here, atomic comparisons in `template_condition_cmp`.

use crate::value::Value;

/// Evaluate a condition that may contain `and`, `or`, and `is defined`.
pub(super) fn evaluate(context: &Value, expression: &str) -> Result<bool, String> {
    let expr = expression.trim();
    if let Some((l, r)) = split_kw(expr, " or ") {
        return Ok(evaluate(context, l)? || evaluate(context, r)?);
    }
    if let Some((l, r)) = split_kw(expr, " and ") {
        return Ok(evaluate(context, l)? && evaluate(context, r)?);
    }
    if let Some(s) = expr.strip_suffix(" is defined") {
        return Ok(is_defined(context, s.trim()));
    }
    if let Some(s) = expr.strip_suffix(" is not defined") {
        return Ok(!is_defined(context, s.trim()));
    }
    super::template_condition_cmp::evaluate(context, expr)
}

/// Split on a keyword surrounded by spaces.
fn split_kw<'a>(expr: &'a str, kw: &str) -> Option<(&'a str, &'a str)> {
    expr.find(kw).map(|p| (&expr[..p], &expr[p + kw.len()..]))
}

/// Whether a key exists with a non-nil value.
fn is_defined(context: &Value, key: &str) -> bool {
    !matches!(
        super::template_context::lookup_value(context, key),
        Ok(Value::Nil) | Err(_)
    )
}
