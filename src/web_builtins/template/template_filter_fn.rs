//! Value-transforming filters.
//!
//! Only filters with unambiguous semantics are implemented. An unknown filter is an error
//! rather than a pass-through, because silently ignoring `| json` would emit a bare value
//! where a page expects valid JSON and break the script consuming it.
//!
//! Application-specific filters (the reference has `clean_llm_meta`, `t`, `form`) are not
//! built in. They belong to the application, so a script registers them by transforming
//! its context before rendering rather than by extending this list — which keeps the
//! engine's behaviour the same everywhere.

use std::rc::Rc;

use super::template_escape::{escape, escape_attr};
use super::template_filter::Filter;
use crate::json;
use crate::value::Value;

/// Apply one named filter to a value.
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
        // Encodes for embedding in a <script> block, which is why the reference always
        // pairs it with `safe`.
        "json" | "json_encode" | "to_json" => json::encode(&value),
        "length" => super::template_filter_len::length_of(&value),
        "upper" => text_map(&value, str::to_uppercase),
        "lower" => text_map(&value, str::to_lowercase),
        "trim" => text_map(&value, |text| text.trim().to_string()),
        // Explicit escaping, for a value already marked `safe` upstream.
        "escape" => text_map(&value, escape),
        "html_attribute_encode" => text_map(&value, escape_attr),
        "int" | "float" | "str" => super::template_filter_len::coerce(name, &value),
        "first" | "last" | "round" | "truncate" => {
            super::template_filter_list::call(name, &value, filter.argument)
        }
        "date" => super::template_filter_date::call(&value, filter.argument),
        other => Err(format!(
            "template: unknown filter `{other}` (have: safe, default, json, to_json, length, \
             upper, lower, trim, escape, html_attribute_encode, int, float, str, first, last, \
             round, truncate, date); application filters belong in the context, computed \
             before rendering. Argument was `{}`",
            filter.argument
        )),
    }
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
