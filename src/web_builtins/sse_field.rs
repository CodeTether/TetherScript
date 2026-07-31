//! Per-field encoding rules for `text/event-stream`.
//!
//! Split from [`super::frame`] so frame assembly and field validation stay
//! separate concerns and each file stays inside the 50-line limit.

use crate::value::Value;

/// Render `field: value\n`, rejecting embedded newlines.
///
/// # Errors
///
/// Returns an error when `value` is not a str, or contains CR or LF: either would
/// forge a field boundary and let a caller inject arbitrary SSE fields.
pub(super) fn single_line(field: &str, value: &Value) -> Result<String, String> {
    let Value::Str(text) = value else {
        return Err(format!(
            "sse_event: {field} must be str, got {}",
            value.type_name()
        ));
    };
    if text.contains('\n') || text.contains('\r') {
        return Err(format!(
            "sse_event: {field} must be a single line; a newline would forge a field boundary"
        ));
    }
    Ok(format!("{field}: {text}\n"))
}

/// Render `retry: <ms>\n`.
///
/// # Errors
///
/// Returns an error naming the field when `value` is not an integer, since the
/// spec requires the reconnection time to be a whole number of milliseconds.
pub(super) fn retry_line(value: &Value) -> Result<String, String> {
    match value {
        Value::Int(ms) => Ok(format!("retry: {ms}\n")),
        other => Err(format!(
            "sse_event: retry must be an int number of milliseconds, got {}",
            other.type_name()
        )),
    }
}

/// Emit one `data:` line per input line.
///
/// Normalizes CRLF and CR to LF first, so a Windows-authored payload cannot leave
/// a stray carriage return inside a frame. An empty payload still emits one empty
/// `data:` line, which is a valid event carrying an empty message.
pub(super) fn data_lines(value: &Value) -> String {
    let text = match value {
        Value::Str(text) => (**text).clone(),
        other => format!("{other}"),
    };
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    normalized
        .split('\n')
        .map(|line| format!("data: {line}\n"))
        .collect()
}
