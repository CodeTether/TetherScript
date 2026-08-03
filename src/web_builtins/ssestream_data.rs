//! Encoding of the multi-line `data` payload for `text/event-stream`.
//!
//! This is the single most-broken part of hand-rolled SSE, so it owns a file.
//! The wire grammar terminates a field at the first LF, which means a payload
//! written as one `data: {json}` line where the JSON is pretty-printed is parsed
//! as a *truncated* event: the client keeps the first physical line, silently
//! discards the rest, and hands the application invalid JSON. There is no error
//! anywhere — the stream stays open and the data is simply wrong.
//!
//! The fix is mechanical: one `data:` prefix per line of the payload. The client
//! rejoins them with `\n`, so the round trip is lossless.
//!
//! # Newline normalization
//!
//! The wire format is LF-delimited. A CRLF-authored payload would otherwise leave
//! a stray CR as the last byte of a field value, which the client keeps as data,
//! and a lone CR would not split a line at all. Both are normalized to LF before
//! splitting, so `"a\r\nb"`, `"a\rb"`, and `"a\nb"` all produce the same two
//! `data:` lines.
//!
//! # Examples
//!
//! ```text
//! "hello"        -> "data: hello\n"
//! "one\ntwo"     -> "data: one\ndata: two\n"
//! "one\r\ntwo"   -> "data: one\ndata: two\n"
//! ""             -> "data: \n"
//! ```

use crate::value::Value;

/// Rewrite CRLF and lone CR as LF.
///
/// # Arguments
///
/// * `text` — Payload text in any newline convention.
///
/// # Returns
///
/// The same text with every line terminator expressed as a single LF. CRLF is
/// replaced first, so a CRLF does not become two blank-separated lines.
fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Render a payload as one `data:` line per line of input.
///
/// # Arguments
///
/// * `value` — Payload. A str is used verbatim; any other value is rendered via
///   its `Display` form first, so an int or a map can be sent without the caller
///   converting it.
///
/// # Returns
///
/// One or more `data: ...\n` lines and never a blank line: the frame terminator
/// is the caller's job, not this function's. An empty payload still yields
/// exactly one `data: \n` line, a valid event carrying the empty string.
pub(super) fn lines(value: &Value) -> String {
    let text = match value {
        Value::Str(text) => (**text).clone(),
        other => format!("{other}"),
    };
    normalize(&text)
        .split('\n')
        .map(|line| format!("data: {line}\n"))
        .collect()
}
