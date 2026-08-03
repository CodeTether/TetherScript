//! `and` / `or` / `is defined` support for `{% if %}` conditions.
//!
//! The reference's views use boolean combinators heavily: 22 conditions in one view alone. Without
//! them, `{% if total_reviews is defined and total_reviews > 0 %}` tries to compare the whole
//! string against `>` and fails.

use super::template_condition::evaluate_atomic;
use crate::value::Value;

/// Evaluate a condition that may contain `and`, `or`, and `is defined`.
///
/// Split on `or` first (lowest precedence), then `and`, then delegate atomic conditions to the
/// existing evaluator. `is defined` is handled inline since it is a presence test rather than a
/// truthiness one: `{% if x is defined %}` must be true when `x` exists with value 0, where
/// `{% if x %}` would be false.
///
/// # Errors
///
/// Returns an error from any atomic condition that fails to evaluate.
pub(super) fn evaluate(context: &Value, expression: &str) -> Result<bool, String> {
    let expr = expression.trim();

    // Split on ` or ` (lowest precedence). Quoted strings are not a concern here: a `|` inside
    // quotes would be a filter pipe, and `or` as a substring of a word like "format" does not match
    // because we require spaces around the operator.
    if let Some((left, right)) = split_keyword(expr, " or ") {
        return Ok(evaluate(context, left)? || evaluate(context, right)?);
    }

    // Split on ` and `.
    if let Some((left, right)) = split_keyword(expr, " and ") {
        return Ok(evaluate(context, left)? && evaluate(context, right)?);
    }

    // `is defined` / `is not defined`.
    if let Some(subject) = expr.strip_suffix(" is defined") {
        return Ok(is_defined(context, subject.trim()));
    }
    if let Some(subject) = expr.strip_suffix(" is not defined") {
        return Ok(!is_defined(context, subject.trim()));
    }

    // Atomic condition: delegate to the existing comparator.
    evaluate_atomic(context, expr)
}

/// Split on a keyword that must be surrounded by spaces, returning the first occurrence.
fn split_keyword<'a>(expr: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
    let mut search_from = 0usize;
    while let Some(pos) = expr[search_from..].find(keyword) {
        let abs = search_from + pos;
        // Ensure it's not inside quotes by checking we're not between unbalanced quotes.
        if !is_in_quotes(expr, abs) {
            return Some((&expr[..abs], &expr[abs + keyword.len()..]));
        }
        search_from = abs + 1;
    }
    None
}

/// Whether position `pos` is inside a single- or double-quoted string.
fn is_in_quotes(expr: &str, pos: usize) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    for (i, c) in expr.char_indices() {
        if i >= pos {
            break;
        }
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            _ => {}
        }
    }
    in_single || in_double
}

/// Whether a key exists in the context with a non-nil value.
fn is_defined(context: &Value, key: &str) -> bool {
    match super::template_context::lookup_value(context, key) {
        Ok(Value::Nil) | Err(_) => false,
        _ => true,
    }
}
