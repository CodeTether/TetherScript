//! `range()` parsing and argument resolution.

use crate::value::Value;

pub(super) struct RangeArgs {
    pub(super) start: i64,
    pub(super) end: i64,
    pub(super) step: i64,
}

pub(super) fn parse_args(inner: &str, context: &Value, lenient: bool) -> Result<RangeArgs, String> {
    let mut start = 0i64;
    let mut end: Option<i64> = None;
    let mut step = 1i64;
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (key, value) = part
            .split_once('=')
            .ok_or_else(|| format!("template: range argument `{part}` must be `key=value`"))?;
        let resolved = resolve_value(value.trim(), context, lenient)?;
        match key.trim() {
            "start" => start = resolved,
            "end" => end = Some(resolved),
            "step" => step = resolved,
            other => return Err(format!("template: range has no argument `{other}`")),
        }
    }
    let end = end.ok_or("template: range requires `end=N`")?;
    if step == 0 {
        return Err("template: range `step` must not be zero".into());
    }
    Ok(RangeArgs { start, end, step })
}

/// Resolve `5 - stars_shown` or `average_rating_rounded | default(value=5)`.
fn resolve_value(expr: &str, context: &Value, lenient: bool) -> Result<i64, String> {
    if let Some((l, r)) = expr.split_once(" - ") {
        return Ok(term(l.trim(), context, lenient)? - term(r.trim(), context, lenient)?);
    }
    term(expr, context, lenient)
}

fn term(expr: &str, context: &Value, lenient: bool) -> Result<i64, String> {
    if let Ok(n) = expr.parse::<i64>() {
        return Ok(n);
    }
    let val = super::template_emit_default::value_of(expr, context).or_else(|_| {
        if lenient {
            Ok(Value::Nil)
        } else {
            Err::<Value, String>("unresolved".into())
        }
    })?;
    match val {
        Value::Int(n) => Ok(n),
        Value::Float(n) => Ok(n as i64),
        Value::Nil if lenient => Ok(0),
        other => Err(format!(
            "range argument is {}, not a number",
            other.type_name()
        )),
    }
}
