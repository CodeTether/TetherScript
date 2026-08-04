//! Collection, rounding, and truncation filters.

use std::rc::Rc;

use crate::value::Value;

pub(super) fn call(name: &str, value: &Value, argument: &str) -> Result<Value, String> {
    match name {
        "first" => end_of(value, true),
        "last" => end_of(value, false),
        "round" => round(value),
        "truncate" => truncate(value, argument),
        _ => Err(format!("template: unknown filter `{name}`")),
    }
}

fn end_of(value: &Value, first: bool) -> Result<Value, String> {
    let Value::List(items) = value else {
        return Err(format!(
            "template: `first`/`last` need a list, got {}",
            value.type_name()
        ));
    };
    let items = items.borrow();
    Ok(if first {
        items.first().cloned().unwrap_or(Value::Nil)
    } else {
        items.last().cloned().unwrap_or(Value::Nil)
    })
}

fn round(value: &Value) -> Result<Value, String> {
    match value {
        Value::Float(n) => Ok(Value::Int(n.round() as i64)),
        Value::Int(_) => Ok(value.clone()),
        other => Err(format!(
            "template: `round` needs a number, got {}",
            other.type_name()
        )),
    }
}

fn truncate(value: &Value, argument: &str) -> Result<Value, String> {
    let Value::Str(text) = value else {
        return Err(format!(
            "template: `truncate` needs a str, got {}",
            value.type_name()
        ));
    };
    let (length, suffix) = super::template_filter_truncate_args::parse(argument)?;
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= length {
        return Ok(Value::Str(Rc::clone(text)));
    }
    let mut out: String = chars[..length].iter().collect();
    out.push_str(&suffix);
    Ok(Value::Str(Rc::new(out)))
}
