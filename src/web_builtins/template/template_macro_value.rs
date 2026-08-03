//! Argument-expression evaluation for a macro call.
//!
//! One concern: turning the text on the right of `name=` into a value. A quoted, numeric,
//! or boolean expression is a literal; anything else is a context key. That is Tera's rule
//! and it is why `row(cfg=item)` passes the loop variable while `row(cfg="item")` passes
//! the four-character string.
//!
//! Literal conversion is delegated to [`super::template_filter_arg::literal_of`], the same
//! function `default(value=...)` uses, so the two spellings cannot drift apart.

use super::template_context::lookup_value;
use super::template_filter_arg::literal_of;
use crate::value::Value;

/// Evaluate one argument expression against the caller's context.
///
/// # Arguments
///
/// * `expression` — Raw text to the right of `=`, already trimmed.
/// * `context` — Caller context, consulted only for the key form.
/// * `path` — Call path, used to name the offender in an error.
/// * `name` — Argument keyword, used to name the offender in an error.
///
/// # Returns
///
/// The literal value, or whatever the key resolves to.
///
/// # Errors
///
/// Returns an error naming the macro, the argument, and the key when a key form does not
/// resolve, or when the expression is empty. A silent nil would render a blank where
/// content belongs.
///
/// # Examples
///
/// ```text
/// resolve("\"new\"", ctx, "ui::badge", "kind")  →  Str("new")
/// resolve("item.title", ctx, "ui::row", "cfg")  →  whatever the context holds
/// ```
pub(super) fn resolve(
    expression: &str,
    context: &Value,
    path: &str,
    name: &str,
) -> Result<Value, String> {
    if expression.is_empty() {
        return Err(format!(
            "template: argument `{name}` to macro `{path}` has no value"
        ));
    }
    if is_literal(expression) {
        return Ok(literal_of(expression));
    }
    lookup_value(context, expression)
        .map_err(|error| format!("template: argument `{name}` to macro `{path}`: {error}"))
}

/// Whether an expression is a self-contained literal rather than a context key.
///
/// A number is only recognized when it *starts* like one. Testing `parse::<f64>()` alone
/// would classify the perfectly ordinary context key `nan` as a float literal.
fn is_literal(expression: &str) -> bool {
    if expression.starts_with(['"', '\'']) || matches!(expression, "true" | "false") {
        return true;
    }
    let numeric = expression.starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '+');
    numeric && (expression.parse::<i64>().is_ok() || expression.parse::<f64>().is_ok())
}
