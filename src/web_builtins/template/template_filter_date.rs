//! `date(format="...")` filter.
//!
//! Formats Unix seconds with the strftime specifiers the reference views use. The civil
//! calendar conversion is duplicated here rather than shared with the `datetime` group,
//! whose helpers are `pub(super)` to a different parent; widening their visibility for one
//! caller would be a worse trade than a small, tested duplicate.

use std::rc::Rc;

use crate::value::Value;

/// Format a timestamp with a strftime-style pattern.
///
/// # Errors
///
/// Returns an error for a non-integer value or a missing `format=` argument.
pub(super) fn call(value: &Value, argument: &str) -> Result<Value, String> {
    let seconds = match value {
        Value::Int(number) => *number,
        Value::Float(number) => *number as i64,
        other => {
            return Err(format!(
                "template: `date` needs Unix seconds as a number, got {}",
                other.type_name()
            ))
        }
    };
    let pattern = pattern_of(argument)?;
    Ok(Value::Str(Rc::new(
        super::template_filter_strftime::render(seconds, &pattern),
    )))
}

/// Extract `format="..."` from the argument list.
///
/// # Errors
///
/// Returns an error when `format` is absent: a default would silently render the wrong
/// shape on every page using the filter.
fn pattern_of(argument: &str) -> Result<String, String> {
    for part in super::template_filter_split::split_outside_quotes(argument, ',') {
        let part = part.trim();
        let Some((key, raw)) = part.split_once('=') else {
            continue;
        };
        if key.trim() == "format" {
            let raw = raw.trim();
            let unquoted = raw
                .strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
                .or_else(|| raw.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')));
            return Ok(unquoted.unwrap_or(raw).to_string());
        }
    }
    Err("template: `date` needs `format=\"...\"`".to_string())
}
