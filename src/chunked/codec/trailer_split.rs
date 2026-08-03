//! Splitting one incoming trailer line into a name/value pair.
//!
//! Separated from the section loop so the field grammar is one testable responsibility.
//!
//! The split is on the *first* colon, and no space is permitted before it. `X-A : 1` is
//! rejected rather than trimmed: RFC 9112 §5.1 requires a recipient to reject whitespace
//! between a field name and its colon precisely because tolerant and strict parsers derive
//! different field names from it, and disagreeing on a field name is how a smuggled
//! `Transfer-Encoding` slips past a front end.
//!
//! # Panics
//!
//! None. The split index comes from `position`, so `..at` and `at + 1..` are both in range;
//! UTF-8 validation is fallible rather than lossy.

use super::super::error::ChunkedError;

/// Split a trailer line into a lowercased name and a trimmed value.
///
/// # Arguments
///
/// * `line` — One non-empty trailer line with its CRLF already removed.
///
/// # Returns
///
/// The lowercased field name and the value with optional leading/trailing whitespace
/// removed.
///
/// # Errors
///
/// [`ChunkedError::Malformed`] if the line has no colon, the name is empty or has
/// whitespace before the colon, or either half is not valid UTF-8.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::split_trailer_field;
///
/// assert_eq!(
///     split_trailer_field(b"X-Sum:  9  ").unwrap(),
///     ("x-sum".to_string(), "9".to_string())
/// );
/// assert!(split_trailer_field(b"no-colon").is_err());
/// assert!(split_trailer_field(b"X-A : 1").is_err());
/// assert!(split_trailer_field(b": 1").is_err());
/// ```
pub fn split_field(line: &[u8]) -> Result<(String, String), ChunkedError> {
    let at = line
        .iter()
        .position(|byte| *byte == b':')
        .ok_or_else(|| ChunkedError::malformed("trailer line has no colon"))?;
    let (raw_name, rest) = (&line[..at], &line[at + 1..]);
    if raw_name.is_empty() || raw_name.last().is_some_and(u8::is_ascii_whitespace) {
        return Err(ChunkedError::malformed(
            "trailer name is empty or padded before its colon",
        ));
    }
    let name = text(raw_name, "trailer name")?.to_ascii_lowercase();
    let value = text(rest, "trailer value")?.trim().to_string();
    Ok((name, value))
}

/// Decode `bytes` as UTF-8, naming `label` on failure.
fn text(bytes: &[u8], label: &str) -> Result<String, ChunkedError> {
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|error| ChunkedError::malformed(format!("{label} is not valid UTF-8: {error}")))
}
