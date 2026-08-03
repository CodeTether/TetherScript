//! Literal-text to [`Value`] conversion for macro defaults and argument values.
//!
//! Kept as raw text at collection time and converted only at bind time, mirroring the
//! engine's `template_filter_arg::literal_of` so that `size="sm"` in a macro header and
//! `default(value="sm")` in a filter behave identically. A quoted literal is always a
//! string; otherwise the most specific type wins so `count=1` binds an int, not `"1"`.

use std::rc::Rc;

use crate::value::Value;

/// Convert literal source text to a value, preferring the most specific type.
///
/// # Arguments
///
/// * `text` — Trimmed literal text, such as `"sm"`, `'sm'`, `12`, `1.5`, or `true`.
///
/// # Returns
///
/// [`Value::Str`] for a quoted literal (quotes stripped), [`Value::Bool`] for `true`
/// or `false`, [`Value::Int`] or [`Value::Float`] for a number, and [`Value::Str`] of
/// the text verbatim as the fallback.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::literal::literal_of;
/// use tetherscript::value::Value;
///
/// assert!(matches!(literal_of("\"sm\""), Value::Str(s) if *s == "sm"));
/// assert!(matches!(literal_of("12"), Value::Int(12)));
/// assert!(matches!(literal_of("true"), Value::Bool(true)));
/// ```
pub fn literal_of(text: &str) -> Value {
    let unquoted = strip_quotes(text);
    if let Some(inner) = unquoted {
        return Value::Str(Rc::new(inner.to_string()));
    }
    match text {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => number_of(text),
    }
}

/// Whether `text` is a quoted literal, and its interior if so.
///
/// # Arguments
///
/// * `text` — Trimmed candidate literal.
///
/// # Returns
///
/// `Some(interior)` for a matched single- or double-quoted literal, else `None`.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::literal::strip_quotes;
///
/// assert_eq!(strip_quotes("'hi'"), Some("hi"));
/// assert_eq!(strip_quotes("hi"), None);
/// ```
pub fn strip_quotes(text: &str) -> Option<&str> {
    text.strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .or_else(|| text.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')))
}

/// Parse an int, then a float, falling back to a bare string.
fn number_of(text: &str) -> Value {
    if let Ok(number) = text.parse::<i64>() {
        return Value::Int(number);
    }
    if let Ok(number) = text.parse::<f64>() {
        return Value::Float(number);
    }
    Value::Str(Rc::new(text.to_string()))
}
