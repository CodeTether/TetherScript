//! Rule dispatch for a single field.
//!
//! Split so this file owns "what does this rule mean" while
//! [`super::validate_fields`] owns "which fields were checked" and
//! [`super::validate_length`] owns the bounded rules.

use crate::value::Value;

use super::validate_length::length;
use super::validate_scan::{is_digits, is_email, is_slug};

/// Apply one rule to one field value.
///
/// # Arguments
///
/// * `field` — Field name, used in the returned message.
/// * `rule` — Rule name: `required`, `min_len`, `max_len`, `email`, `digits`, `slug`.
/// * `spec` — Rule argument; only the length rules read it.
/// * `value` — The submitted value, or `None` when the field was absent.
///
/// # Returns
///
/// `Ok(None)` when the field satisfies the rule, `Ok(Some(message))` when it
/// fails. A failing field is an ordinary outcome, not an error.
///
/// # Errors
///
/// Returns an error naming the rule when it is unknown, or when a length rule was
/// given a non-integer bound. An unknown rule is a program bug, so treating it as
/// a pass would hide the mistake exactly where validation matters.
pub(super) fn apply(
    field: &str,
    rule: &str,
    spec: &Value,
    value: Option<&Value>,
) -> Result<Option<String>, String> {
    // Only `required` cares about absence; every other rule skips a missing field
    // so optional inputs need no extra flag.
    let text = value.map(render);
    if rule == "required" {
        let present = text.as_deref().is_some_and(|body| !body.trim().is_empty());
        return Ok((!present).then(|| format!("{field} is required")));
    }
    let Some(text) = text else {
        return Ok(None);
    };

    match rule {
        "min_len" => length(field, &text, spec, true),
        "max_len" => length(field, &text, spec, false),
        "email" => Ok((!is_email(&text)).then(|| format!("{field} must be a valid email address"))),
        "digits" => Ok((!is_digits(&text)).then(|| format!("{field} must contain only digits"))),
        "slug" => Ok((!is_slug(&text)).then(|| format!("{field} must be a lowercase slug"))),
        other => Err(format!("validate_fields: unknown rule `{other}`")),
    }
}

/// Render a submitted value as text so numeric input validates like form input.
fn render(value: &Value) -> String {
    match value {
        Value::Str(text) => (**text).clone(),
        Value::Nil => String::new(),
        other => format!("{other}"),
    }
}
