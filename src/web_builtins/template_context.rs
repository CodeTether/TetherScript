//! Context lookup for template placeholders.
//!
//! An unknown key is a named error, never an empty string. A typo like
//! `{{ user.nmae }}` would otherwise render a blank where content belongs and
//! ship a silently broken page; failing loudly turns that into a caught bug.

use crate::value::Value;

/// Resolve a possibly-dotted key against the context map.
///
/// # Arguments
///
/// * `context` — The root context, which must be a map.
/// * `key` — A bare name such as `title`, or a dotted path such as `user.name`.
///
/// # Returns
///
/// The rendered text for the value. Strings render as-is; ints, floats, and bools
/// render through their display form; `nil` renders as the empty string, because
/// a present-but-null field is a legitimate blank.
///
/// # Errors
///
/// Returns an error naming the missing segment and the full key when a segment is
/// absent, when a non-final segment is not a map, or when the value is a list or
/// function with no sensible text form.
pub(super) fn lookup(context: &Value, key: &str) -> Result<String, String> {
    let mut current = context.clone();
    for segment in key.split('.') {
        let Value::Map(map) = &current else {
            return Err(format!(
                "template_render: cannot look up `{segment}` in `{key}`: parent is {}, not a map",
                current.type_name()
            ));
        };
        let next = map.borrow().get(segment).cloned();
        current = match next {
            Some(value) => value,
            None => {
                return Err(format!(
                    "template_render: unknown key `{segment}` in `{key}`"
                ))
            }
        };
    }
    render_scalar(&current, key)
}

/// Convert a resolved value to its rendered text.
fn render_scalar(value: &Value, key: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        Value::Int(_) | Value::Float(_) | Value::Bool(_) => Ok(format!("{value}")),
        Value::Nil => Ok(String::new()),
        other => Err(format!(
            "template_render: key `{key}` is {}, which has no text form",
            other.type_name()
        )),
    }
}
