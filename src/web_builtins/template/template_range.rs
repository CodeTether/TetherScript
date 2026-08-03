//! `range()` support for `{% for i in range(end=N) %}`.

use crate::value::Value;

pub(super) fn is_range(subject: &str) -> bool {
    subject.trim().starts_with("range(")
}

/// Evaluate a `range(...)` call to a list of Int values.
///
/// The subject may contain filters inside the parens:
/// `range(end=average_rating_rounded | default(value=5))`. The range parser extracts the inner
/// content, resolves each argument through the value pipeline (so `| default(value=..)` works),
/// and builds the integer sequence.
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
