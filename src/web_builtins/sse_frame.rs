//! Frame assembly for `text/event-stream`.
//!
//! Split from [`super::sse`] so registration and formatting stay separate.
//!
//! Field order is fixed as `event`, `id`, `retry`, then `data`, because `data` is
//! the only multi-line field and keeping it last makes a truncated frame obvious
//! when reading a capture.

use std::rc::Rc;

use crate::value::Value;

#[path = "sse_field.rs"]
mod field;

/// Build one complete event frame from a map of fields.
///
/// # Arguments
///
/// * `fields` — Map that may carry `data`, `event`, `id`, and `retry`. Unknown
///   keys are ignored, so a caller can pass a richer record straight through.
///
/// # Returns
///
/// The frame, always terminated by a blank line so the client dispatches it.
///
/// # Errors
///
/// Returns an error when `fields` is not a map, when `event` or `id` contains a
/// newline (which would forge a field boundary), when `id` contains NUL, or when
/// `retry` is not an integer.
pub(super) fn event(fields: &Value) -> Result<Value, String> {
    let Value::Map(map) = fields else {
        return Err(format!(
            "sse_event: fields must be a map, got {}",
            fields.type_name()
        ));
    };
    let map = map.borrow();
    let mut out = String::new();

    if let Some(value) = map.get("event") {
        out.push_str(&field::single_line("event", value)?);
    }
    if let Some(value) = map.get("id") {
        let line = field::single_line("id", value)?;
        // The spec forbids NUL in the last-event-ID buffer.
        if line.contains('\0') {
            return Err("sse_event: id must not contain NUL".into());
        }
        out.push_str(&line);
    }
    if let Some(value) = map.get("retry") {
        out.push_str(&field::retry_line(value)?);
    }
    if let Some(value) = map.get("data") {
        out.push_str(&field::data_lines(value));
    }

    // The blank line is what dispatches the event.
    out.push('\n');
    Ok(Value::Str(Rc::new(out)))
}

/// Build a `:`-prefixed comment line, used as a keep-alive.
///
/// # Arguments
///
/// * `text` — Comment body. Must be a single line.
///
/// # Returns
///
/// One line of the form `": <text>\n"`. A comment is not an event, so it carries
/// no blank-line terminator and dispatches nothing.
///
/// # Errors
///
/// Returns an error when `text` is not a str or spans multiple lines.
pub(super) fn comment(text: &Value) -> Result<Value, String> {
    let line = field::single_line("comment", text)?;
    // Reuse the field renderer, then swap `comment: ` for the bare `: ` prefix.
    let body = line.strip_prefix("comment: ").unwrap_or(&line);
    Ok(Value::Str(Rc::new(format!(": {body}"))))
}

/// Build a bare `retry:` frame.
///
/// # Arguments
///
/// * `ms` — Reconnection delay in whole milliseconds.
///
/// # Returns
///
/// A `retry:` frame terminated by a blank line.
///
/// # Errors
///
/// Returns an error naming the field when `ms` is not an integer.
pub(super) fn retry(ms: &Value) -> Result<Value, String> {
    let line = field::retry_line(ms)?;
    Ok(Value::Str(Rc::new(format!("{line}\n"))))
}
