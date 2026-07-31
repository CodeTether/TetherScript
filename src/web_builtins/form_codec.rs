//! Percent-encoding and percent-decoding per RFC 3986.
//!
//! Split from the registration layer so the codec is testable as plain Rust and
//! so each file stays inside the 50-line limit. `+` is treated as an encoded
//! space on the way in, which is the `application/x-www-form-urlencoded`
//! convention rather than generic URI syntax.

use super::form_hex;

/// Percent-encode `input`, leaving only the RFC 3986 unreserved set intact.
///
/// The unreserved set is ALPHA / DIGIT / `-` / `.` / `_` / `~`. Every other byte,
/// including each byte of a multi-byte UTF-8 sequence, becomes `%XX` in uppercase
/// hex. Space becomes `%20`, not `+`, which is valid in both a query string and a
/// form body.
///
/// # Arguments
///
/// * `input` — Text to encode.
///
/// # Returns
///
/// The encoded string.
pub(crate) fn encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(*byte as char);
        } else {
            out.push('%');
            out.push(form_hex::digit(byte >> 4));
            out.push(form_hex::digit(byte & 0x0f));
        }
    }
    out
}

/// Decode `%XX` escapes, and `+` as space.
///
/// # Arguments
///
/// * `input` — Encoded text.
/// * `label` — Error prefix, so each caller names its own built-in.
///
/// # Errors
///
/// Returns an error naming the offending sequence when an escape is truncated or
/// contains a non-hex digit, or when the decoded bytes are not valid UTF-8.
pub(crate) fn decode(input: &str, label: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => out.push(b' '),
            b'%' => {
                out.push(escape_at(bytes, index, label)?);
                index += 2;
            }
            byte => out.push(byte),
        }
        index += 1;
    }
    String::from_utf8(out).map_err(|_| format!("{label}: decoded bytes are not valid UTF-8"))
}

/// Read the two hex digits of the `%XX` escape beginning at `index`.
fn escape_at(bytes: &[u8], index: usize, label: &str) -> Result<u8, String> {
    let pair = bytes.get(index + 1..index + 3).ok_or_else(|| {
        format!(
            "{label}: truncated percent escape `{}`",
            String::from_utf8_lossy(&bytes[index..])
        )
    })?;
    match (form_hex::value(pair[0]), form_hex::value(pair[1])) {
        (Some(high), Some(low)) => Ok(high << 4 | low),
        _ => Err(format!(
            "{label}: invalid percent escape `%{}`",
            String::from_utf8_lossy(pair)
        )),
    }
}
