//! Value type tags for the session-store wire format.
//!
//! # Why values carry a tag
//!
//! A payload map holds ints (a user id), bools (an entitlement flag), floats, nil,
//! and strings. If everything were written as text, decoding would have to *guess*:
//! `"42"` would come back as an int and a zip code of `"01234"` as `1234`. Round-trip
//! must be exact, so one leading character records the original type and the
//! decoder never guesses.
//!
//! Nested lists and maps are deliberately rejected rather than flattened. This is a
//! flat one-hash-per-session format; silently stringifying a list would break the
//! round-trip guarantee that the rest of this group rests on. A caller with nested
//! data encodes it with `json_encode` and stores the result as a str.

use crate::value::Value;

/// Render one payload value as `<tag><text>`.
///
/// # Arguments
///
/// * `label` — Built-in and parameter name, used verbatim in the error.
/// * `key` — Map key being written, named in the error.
/// * `value` — Value to render.
///
/// # Returns
///
/// The tagged, *unescaped* text. Escaping is the caller's step.
///
/// # Errors
///
/// Returns a named error for a value that is not str, int, float, bool, or nil.
///
/// # Examples
///
/// ```rust,ignore
/// # use crate::value::Value;
/// assert_eq!(tagged("l", "k", &Value::Int(42)).unwrap(), "i42");
/// ```
pub(super) fn tagged(label: &str, key: &str, value: &Value) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok(format!("s{text}")),
        Value::Int(number) => Ok(format!("i{number}")),
        Value::Float(number) => Ok(format!("f{number:?}")),
        Value::Bool(flag) => Ok(format!("b{flag}")),
        Value::Nil => Ok("n".to_string()),
        other => Err(format!(
            "{label}: value for key {key:?} must be str, int, float, bool, or nil, got {}",
            other.type_name()
        )),
    }
}
