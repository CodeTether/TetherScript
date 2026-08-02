//! Named-template lookup for inheritance.
//!
//! Templates are supplied by the caller as a map rather than read from disk:
//! `template_render` is a pure built-in, so reading files from inside it would
//! bypass the `fs` capability entirely. A host that wants filesystem templates
//! reads them through `fs` and passes the map in.

use crate::value::Value;

/// Look up a named template in the caller-supplied map.
///
/// # Arguments
///
/// * `templates` — Map of name to source, or `Value::Nil` when none was supplied.
/// * `name` — Template name as written in `{% extends %}`.
///
/// # Errors
///
/// Returns an error naming the template when it is absent, because a missing parent
/// would otherwise render as a blank page.
pub(super) fn source_of(templates: &Value, name: &str) -> Result<String, String> {
    if matches!(templates, Value::Nil) {
        return Err(format!(
            "template: `{name}` is extended but no templates were supplied; \
             use template_render_inherited(template, context, templates)"
        ));
    }
    let Value::Map(map) = templates else {
        return Err(format!(
            "template: templates must be a map of name to source, got {}",
            templates.type_name()
        ));
    };
    match map.borrow().get(name) {
        Some(Value::Str(text)) => Ok((**text).clone()),
        Some(other) => Err(format!(
            "template: template `{name}` must be str, got {}",
            other.type_name()
        )),
        None => Err(format!("template: unknown template `{name}`")),
    }
}
