//! Parsing the ASCII integers that RESP uses for lengths and integer replies.
//!
//! RESP never sends binary numbers; every count is decimal ASCII. A malformed one
//! is a framing failure, not a value, so it becomes
//! [`RedisError::Protocol`].

use super::error::RedisError;

/// Parse a decimal ASCII integer, allowing a leading `-`.
///
/// # Arguments
///
/// * `line` — Line contents with the type byte and CRLF already stripped.
/// * `context` — What was being parsed, for the error message.
///
/// # Returns
///
/// The parsed signed 64-bit value.
///
/// # Errors
///
/// [`RedisError::Protocol`] when the line is empty, is not valid UTF-8, or does
/// not parse as an `i64`. The message quotes what arrived, because a framing
/// desync usually shows up here first and the offending bytes are the clue.
pub(super) fn parse_i64(line: &[u8], context: &str) -> Result<i64, RedisError> {
    let text = std::str::from_utf8(line)
        .map_err(|_| RedisError::Protocol(format!("{context}: length line is not valid UTF-8")))?;
    text.parse::<i64>()
        .map_err(|_| RedisError::Protocol(format!("{context}: `{text}` is not a valid integer")))
}

/// Split an error line into its kind and message.
///
/// # Arguments
///
/// * `line` — Error text with the leading `-` and trailing CRLF removed.
///
/// # Returns
///
/// `(kind, message)`. Redis conventionally leads with an uppercase token such as
/// `ERR` or `WRONGTYPE`; when there is no space the whole line becomes the kind
/// and the message is empty, so nothing is silently dropped.
pub(super) fn split_error(line: &str) -> (String, String) {
    match line.split_once(' ') {
        Some((kind, message)) => (kind.to_string(), message.to_string()),
        None => (line.to_string(), String::new()),
    }
}
