//! Parsing a tagged value back into a [`Value`].
//!
//! The inverse of [`super::sessionstore_tag::tagged`], split out because decoding is
//! the failing direction: the text may come from Redis, where another writer or a
//! truncated write can produce something this format never emits.

use std::rc::Rc;

use crate::value::Value;

/// Parse `<tag><text>` into a value.
///
/// # Arguments
///
/// * `label` — Built-in and parameter name, used verbatim in the error.
/// * `key` — Map key being read, named in the error.
/// * `body` — The already-unescaped tagged text.
///
/// # Returns
///
/// The reconstructed value.
///
/// # Errors
///
/// Returns a named error when `body` is empty, carries an unknown tag, carries a
/// numeric or boolean body that does not parse, or tags nil with a non-empty body.
///
/// # Examples
///
/// ```rust,ignore
/// assert!(matches!(parse("l", "k", "i42"), Ok(crate::value::Value::Int(42))));
/// ```
pub(super) fn parse(label: &str, key: &str, body: &str) -> Result<Value, String> {
    let mut chars = body.chars();
    let tag = chars
        .next()
        .ok_or_else(|| format!("{label}: value for key {key:?} is empty, expected a type tag"))?;
    let rest = chars.as_str();
    match tag {
        's' => Ok(Value::Str(Rc::new(rest.to_string()))),
        'i' => rest
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| bad(label, key, "int", rest)),
        'f' => rest
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| bad(label, key, "float", rest)),
        'b' => rest
            .parse::<bool>()
            .map(Value::Bool)
            .map_err(|_| bad(label, key, "bool", rest)),
        'n' if rest.is_empty() => Ok(Value::Nil),
        // Reported separately rather than folded into "unknown tag": the tag *is*
        // known, the body is wrong, and naming the wrong thing sends a reader hunting
        // for a tag that exists.
        'n' => Err(format!(
            "{label}: value for key {key:?} is tagged nil but carries a body: {rest:?}"
        )),
        other => Err(format!(
            "{label}: value for key {key:?} has unknown type tag {other:?}"
        )),
    }
}

/// Error wording shared by the three parsing tags.
fn bad(label: &str, key: &str, kind: &str, body: &str) -> String {
    format!("{label}: value for key {key:?} is not a valid {kind}: {body:?}")
}
