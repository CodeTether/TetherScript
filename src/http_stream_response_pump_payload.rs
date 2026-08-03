//! What a generator may return, and how it becomes bytes.
//!
//! Split from [`super`] so the loop and the value contract stay separate.
//!
//! The accepted returns are deliberately narrow:
//!
//! | Return | Meaning |
//! | --- | --- |
//! | `nil` | End of stream. |
//! | `str` | Pre-framed payload, written verbatim. |
//! | `bytes` | Pre-framed payload, written verbatim. |
//! | `Ok(v)` | Unwrapped, then re-applied to this table. |
//! | `Err(e)` | Ends the stream, reported as a generator error. |
//!
//! `Ok`/`Err` are accepted because `sse_event`, `sse_comment`, and `sse_retry`
//! all return a `Result`, so `fn() { sse_event(f) }` — without `?` — is the
//! shape a script author writes first. Accepting it avoids a confusing type
//! error for correct-looking code.
//!
//! Framing is **not** applied here. The payload is written exactly as produced,
//! so the existing SSE built-ins remain the single source of truth for the
//! `text/event-stream` grammar and this module cannot double-terminate an event.

use crate::value::{ResultValue, Value};

/// Convert one generator return value into payload bytes.
///
/// # Arguments
///
/// * `produced` — Whatever the generator returned.
///
/// # Returns
///
/// `Ok(Some(bytes))` for a payload to flush, `Ok(None)` to end the stream
/// normally.
///
/// # Errors
///
/// Returns `Err` when the generator returned `Err(e)`, or a value of a type this
/// contract does not accept; the message names the offending type so the author
/// can see that, for example, an int is not a frame.
///
/// # Examples
///
/// ```text
/// bytes(&Value::Nil)                        == Ok(None)
/// bytes(&str_value("data: hi\n\n"))          == Ok(Some(b"data: hi\n\n".to_vec()))
/// bytes(&ok(str_value("data: hi\n\n")))      == Ok(Some(b"data: hi\n\n".to_vec()))
/// bytes(&err("boom")).is_err()               == true
/// bytes(&Value::Int(1)).is_err()             == true
/// ```
pub(crate) fn bytes(produced: &Value) -> Result<Option<Vec<u8>>, String> {
    match produced {
        Value::Nil => Ok(None),
        Value::Str(text) => Ok(Some(text.as_bytes().to_vec())),
        Value::Bytes(raw) => Ok(Some(raw.borrow().clone())),
        Value::Result(result) => match result.as_ref() {
            ResultValue::Ok(inner) => nested(inner),
            ResultValue::Err(error) => Err(error.clone()),
        },
        other => Err(format!(
            "http_serve: stream generator must return str, bytes, result, or nil, got {}",
            other.type_name()
        )),
    }
}

/// Unwrap one level of `Ok`, refusing a nested `Result`.
///
/// # Arguments
///
/// * `inner` — The value inside `Ok`.
///
/// # Returns
///
/// The payload, as [`bytes`] would.
///
/// # Errors
///
/// Returns `Err` for `Ok(Ok(..))`: a doubly-wrapped frame is a script bug, and
/// unwrapping it silently would hide the mistake rather than report it.
fn nested(inner: &Value) -> Result<Option<Vec<u8>>, String> {
    if matches!(inner, Value::Result(_)) {
        return Err("http_serve: stream generator returned a nested result".to_string());
    }
    bytes(inner)
}
