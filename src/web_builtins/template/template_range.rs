//! `range()` support for `{% for i in range(end=N) %}`.
//!
//! The reference's views use `range` to render star ratings: `{% for i in range(end=rating) %}`.

use crate::value::Value;

/// Whether a `for` subject is a `range()` call.
pub(super) fn is_range(subject: &str) -> bool {
    subject.trim().starts_with("range(")
}

/// Evaluate a `range(start=A, end=B, step=S)` call to a list of Int values.
///
/// # Errors
///
/// Returns an error when the call is malformed, `end` is missing, or `step` is zero.
pub(super) fn evaluate(
    subject: &str,
    context: &Value,
    lenient: bool,
) -> Result<Vec<Value>, String> {
    let inner = subject
        .trim()
        .strip_prefix("range(")
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("template: malformed range call `{subject}`"))?;

    let args = super::template_range_args::parse_args(inner, context, lenient)?;
    Ok(build(args.start, args.end, args.step))
}

/// Build the list of integers from `start` (inclusive) to `end` (exclusive), stepping by `step`.
fn build(start: i64, end: i64, step: i64) -> Vec<Value> {
    let mut out = Vec::new();
    let mut current = start;
    if step > 0 {
        while current < end {
            out.push(Value::Int(current));
            current += step;
        }
    } else {
        while current > end {
            out.push(Value::Int(current));
            current += step;
        }
    }
    out
}
