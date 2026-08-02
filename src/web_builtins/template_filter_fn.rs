//! Value-transforming filters.
//!
//! Only filters with unambiguous semantics are implemented. An unknown filter is an
//! error rather than a pass-through, because silently ignoring `| json` would emit a
//! bare value where a page expects valid JSON and break the script consuming it.

use std::rc::Rc;

use super::template_filter::Filter;
use crate::json;
use crate::value::Value;

/// Apply one named filter to a value.
///
/// # Arguments
///
/// * `name` — Filter name.
/// * `current` — Value so far, `None` when the key was absent.
/// * `filter` — The parsed filter, for its argument.
///
/// # Errors
///
/// Returns an error for an unknown filter, or for one applied to a missing value.
pub(super) fn call(
    name: &str,
    current: Option<Value>,
    filter: &Filter<'_>,
) -> Result<Value, String> {
    let value = current.ok_or_else(|| {
        format!("template: `{name}` applied to a missing value; add `| default(value=..)` first")
    })?;
    match name {
        // Encodes for embedding in a <script> block, which is why the reference
        // always pairs it with `safe`.
        "json" | "json_encode" => json::encode(&value),
        "length" => length_of(&value),
        "upper" => text_map(&value, str::to_uppercase),
        "lower" => text_map(&value, str::to_lowercase),
        "trim" => text_map(&value, |text| text.trim().to_string()),
        other => Err(format!(
            "template: unknown filter `{other}` (have: safe, default, json, length, \
             upper, lower, trim); argument was `{}`",
            filter.argument
        )),
    }
}

/// Length of a string, list, or map.
fn length_of(value: &Value) -> Result<Value, String> {
    let length = match value {
        Value::Str(text) => text.chars().count(),
        Value::List(items) => items.borrow().len(),
        Value::Map(entries) => entries.borrow().len(),
        other => {
            return Err(format!(
                "template: `length` needs a str, list, or map, got {}",
                other.type_name()
            ))
        }
    };
    Ok(Value::Int(length as i64))
}

/// Apply a string transform, requiring a string input.
fn text_map(value: &Value, transform: impl Fn(&str) -> String) -> Result<Value, String> {
    match value {
        Value::Str(text) => Ok(Value::Str(Rc::new(transform(text)))),
        other => Err(format!(
            "template: filter needs a str, got {}",
            other.type_name()
        )),
    }
}
