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
        let resolved = resolve_int(value.trim(), context, lenient)?;
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

fn resolve_int(text: &str, context: &Value, lenient: bool) -> Result<i64, String> {
    match text.parse::<i64>() {
        Ok(n) => Ok(n),
        Err(_) => match super::template_context::lookup_value(context, text) {
            Ok(Value::Int(n)) => Ok(n),
            Ok(Value::Float(n)) => Ok(n as i64),
            Ok(Value::Nil) if lenient => Ok(0),
            Ok(other) => Err(format!(
                "template: range argument `{text}` is {}, not a number",
                other.type_name()
            )),
            Err(_) if lenient => Ok(0),
            Err(_) => Err(format!(
                "template: range argument `{text}` is not a number or a known key"
            )),
        },
    }
}
