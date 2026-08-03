//! Resolving an argument expression against the **caller's** context.
//!
//! This is the one place the caller's context is read, and reading it here is correct: the
//! *argument values* are evaluated at the call site, in the caller's scope, before the
//! child scope exists. What must never happen is the macro *body* seeing that context —
//! see [`crate::tmplmacro::bind`].
//!
//! A dotted path such as `cfg.hero.title` walks nested maps. A quoted, numeric, or boolean
//! literal is converted by [`crate::tmplmacro::literal::literal_of`] without any lookup.

use crate::tmplmacro::literal::{literal_of, strip_quotes};
use crate::value::Value;

/// Resolve one argument expression to a value.
///
/// # Arguments
///
/// * `expression` — Raw argument text: a literal, or a dotted context key.
/// * `context` — The **caller's** context.
/// * `path` — Call path, for the error message.
/// * `param` — Parameter being bound, for the error message.
///
/// # Returns
///
/// The resolved value.
///
/// # Errors
///
/// Returns an error naming the macro, the parameter, and the expression when a non-literal
/// expression does not resolve in the caller's context. A silent `Nil` here would ship a
/// blank where content belongs.
///
/// # Panics
///
/// None.
///
/// # Examples
///
/// ```
/// use std::cell::RefCell;
/// use std::collections::HashMap;
/// use std::rc::Rc;
/// use tetherscript::tmplmacro::argvalue::resolve;
/// use tetherscript::value::Value;
///
/// let mut map = HashMap::new();
/// map.insert("title".to_string(), Value::Int(7));
/// let context = Value::Map(Rc::new(RefCell::new(map)));
///
/// assert!(matches!(resolve("title", &context, "p", "a").unwrap(), Value::Int(7)));
/// assert!(matches!(resolve("\"lit\"", &context, "p", "a").unwrap(), Value::Str(_)));
/// assert!(resolve("missing", &context, "p", "a").is_err());
/// ```
pub fn resolve(
    expression: &str,
    context: &Value,
    path: &str,
    param: &str,
) -> Result<Value, String> {
    if strip_quotes(expression).is_some() || is_scalar(expression) {
        return Ok(literal_of(expression));
    }
    lookup(context, expression).ok_or_else(|| {
        format!(
            "template: argument `{param}={expression}` to macro `{path}` \
             does not resolve in the calling context"
        )
    })
}

/// Whether the text is a bare boolean or number literal needing no lookup.
///
/// A digit, sign, or dot is required before trying `f64`, because Rust parses `inf` and
/// `nan` as floats and a context key spelled `inf` must remain a lookup.
fn is_scalar(text: &str) -> bool {
    if matches!(text, "true" | "false") {
        return true;
    }
    let numeric = text.starts_with(|c: char| c.is_ascii_digit() || matches!(c, '-' | '+' | '.'));
    numeric && text.parse::<f64>().is_ok()
}

/// Walk a dotted key path through nested maps.
fn lookup(context: &Value, expression: &str) -> Option<Value> {
    let mut current = context.clone();
    for segment in expression.split('.') {
        let Value::Map(map) = current else {
            return None;
        };
        current = map.borrow().get(segment)?.clone();
    }
    Some(current)
}
