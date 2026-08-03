//! Filter argument parsing.
//!
//! Tera writes arguments as `key=value`, as in `default(value=1)`. Only the `value`
//! key is meaningful for the filters implemented here, so anything else is rejected
//! rather than silently ignored.

use std::rc::Rc;

use crate::value::Value;

/// Parse a `value=<literal>` argument into a tetherscript value.
///
/// # Arguments
///
/// * `argument` — Text inside the filter's parentheses.
///
/// # Returns
///
/// The literal as an int, float, bool, or str.
///
/// # Errors
///
/// Returns an error when the argument is missing, uses an unexpected key, or is
/// empty. A missing argument would otherwise make `default()` a silent no-op.
pub(super) fn parse(argument: &str) -> Result<Value, String> {
    if argument.is_empty() {
        return Err("template: `default` needs an argument, e.g. default(value=0)".into());
    }
    let (key, literal) = argument
        .split_once('=')
        .ok_or_else(|| format!("template: `default({argument})` must be `value=<literal>`"))?;
    if key.trim() != "value" {
        return Err(format!(
            "template: unknown filter argument `{}`; expected `value`",
            key.trim()
        ));
    }
    Ok(literal_of(literal.trim()))
}

/// Convert a literal to a value, preferring the most specific type.
///
/// A quoted literal is always a string; otherwise numbers and booleans are
/// recognized so `default(value=1)` yields an int rather than the text "1".
pub(super) fn literal_of(text: &str) -> Value {
    let unquoted = text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .or_else(|| text.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')));
    if let Some(inner) = unquoted {
        return Value::Str(Rc::new(inner.to_string()));
    }
    match text {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => number_of(text),
    }
}

/// Parse an int, then a float, falling back to a string.
fn number_of(text: &str) -> Value {
    if let Ok(number) = text.parse::<i64>() {
        return Value::Int(number);
    }
    if let Ok(number) = text.parse::<f64>() {
        return Value::Float(number);
    }
    Value::Str(Rc::new(text.to_string()))
}
