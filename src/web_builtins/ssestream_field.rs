//! Validation of the single-line SSE fields: `event`, `id`, and `retry`.
//!
//! Only `data` may span lines. `event`, `id`, and `retry` are single-line
//! fields, so an embedded CR or LF in one of them is not a formatting nuisance —
//! it is a field-injection hole. A caller that passed an attacker-controlled
//! string as `event` could otherwise emit `event: a\ndata: forged` and forge a
//! whole event, so a newline is rejected rather than stripped: silently mangling
//! a value hides the bug, and an error names it.
//!
//! # NUL in `id`
//!
//! The `id` field feeds the client's *last event ID* buffer, which is echoed back
//! on reconnect in the `Last-Event-ID` request header. The HTML spec requires the
//! field to be ignored when it contains a NUL, so an `id` with a NUL would appear
//! to be set while the client silently drops it and replays from an older
//! position after a reconnect. Rejecting it up front is the only honest option.

use crate::value::Value;

/// Render `name: value\n` for a single-line field.
///
/// # Arguments
///
/// * `name` — Field name, used verbatim as the prefix and in error messages.
/// * `value` — Field value. Must be a str.
///
/// # Returns
///
/// The rendered line, LF-terminated.
///
/// # Errors
///
/// Returns an error when `value` is not a str, or when it contains CR or LF,
/// naming the field so the caller can find the offending input.
pub(super) fn line(name: &str, value: &Value) -> Result<String, String> {
    let Value::Str(text) = value else {
        return Err(format!(
            "sse: {name} must be str, got {}",
            value.type_name()
        ));
    };
    if text.contains('\n') || text.contains('\r') {
        return Err(format!(
            "sse: {name} must be a single line; a newline would forge a field boundary"
        ));
    }
    Ok(format!("{name}: {text}\n"))
}

/// Render `id: value\n`, additionally rejecting NUL.
///
/// # Arguments
///
/// * `value` — Event id. Must be a str with no CR, LF, or NUL.
///
/// # Returns
///
/// The rendered `id:` line.
///
/// # Errors
///
/// Returns an error for a non-str, a multi-line value, or a value containing NUL,
/// which the client would silently discard.
pub(super) fn id_line(value: &Value) -> Result<String, String> {
    let rendered = line("id", value)?;
    if rendered.contains('\0') {
        return Err("sse: id must not contain NUL; the client silently ignores such an id".into());
    }
    Ok(rendered)
}

/// Render `retry: <ms>\n`.
///
/// # Arguments
///
/// * `value` — Reconnection delay. Must be an int number of milliseconds.
///
/// # Returns
///
/// The rendered `retry:` line.
///
/// # Errors
///
/// Returns an error naming the field when `value` is not an int, or when it is
/// negative: the spec parses the value as a non-negative integer and ignores the
/// field otherwise, so a negative delay would be silently dropped.
pub(super) fn retry_line(value: &Value) -> Result<String, String> {
    match value {
        Value::Int(ms) if *ms >= 0 => Ok(format!("retry: {ms}\n")),
        Value::Int(ms) => Err(format!(
            "sse: retry must be a non-negative number of milliseconds, got {ms}"
        )),
        other => Err(format!(
            "sse: retry must be an int number of milliseconds, got {}",
            other.type_name()
        )),
    }
}
