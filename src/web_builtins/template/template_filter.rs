//! Filter expression parsing, and argument coercion for the built-ins.
//!
//! Splitting is separate from application so a malformed pipeline is reported before any
//! lookup runs.

use std::rc::Rc;

use super::template_filter_split::split_outside_quotes;
use crate::system::result_value;
use crate::value::Value;

/// One filter in a pipeline: its name and optional `key=value` argument.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Filter<'a> {
    /// Filter name, such as `default` or `safe`.
    pub name: &'a str,
    /// Raw argument text inside the parentheses, empty when there is none.
    pub argument: &'a str,
}

/// Split a hole body into its value key and filter pipeline.
///
/// # Errors
///
/// Returns an error for an empty key or an empty filter name, since `{{ x | }}` is always
/// a typo rather than a shorthand.
pub(super) fn split(body: &str) -> Result<(&str, Vec<Filter<'_>>), String> {
    // Quote-aware, because a `|` inside a filter argument is data rather than a pipeline
    // separator.
    let mut parts = split_outside_quotes(body, '|').into_iter();
    let key = parts.next().unwrap_or("").trim();
    if key.is_empty() {
        return Err(format!("template: no value before `|` in `{body}`"));
    }
    let mut filters = Vec::new();
    for part in parts {
        filters.push(one(part.trim(), body)?);
    }
    Ok((key, filters))
}

/// Parse a single `name` or `name(argument)` filter.
fn one<'a>(text: &'a str, body: &str) -> Result<Filter<'a>, String> {
    let Some(open) = text.find('(') else {
        if text.is_empty() {
            return Err(format!("template: empty filter name in `{body}`"));
        }
        return Ok(Filter {
            name: text,
            argument: "",
        });
    };
    let close = text
        .rfind(')')
        .ok_or_else(|| format!("template: unclosed filter argument in `{body}`"))?;
    Ok(Filter {
        name: text[..open].trim(),
        argument: text[open + 1..close].trim(),
    })
}

/// Coerce a built-in argument to a string, naming the parameter on mismatch.
///
/// # Errors
///
/// Returns an error naming `label` and the actual type.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Wrap a fallible render as a tetherscript `Result`.
pub(super) fn wrap(result: Result<String, String>) -> Value {
    result_value(result.map(|text| Value::Str(Rc::new(text))))
}
