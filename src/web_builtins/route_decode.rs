//! Percent-decoding for captured path segments.
//!
//! A capture is decoded so `/customers/a%20b` yields `a b`. Decoding happens
//! *after* segmentation, never before: decoding first would turn an encoded `%2F`
//! into a real separator and let a single `{name}` capture appear to span two
//! segments, which is a path-traversal shaped bug.
//!
//! `+` is left alone. It means space in a query string, not in a path, and
//! rewriting it here would corrupt identifiers that legitimately contain `+`.

/// Percent-decode one path segment.
///
/// # Arguments
///
/// * `segment` — A single path segment, already split on `/`.
///
/// # Returns
///
/// The decoded text.
///
/// # Errors
///
/// Returns an error naming the offending escape when a `%` is truncated, its
/// digits are not hex, or the decoded bytes are not valid UTF-8.
pub(super) fn decode(segment: &str) -> Result<String, String> {
    if !segment.contains('%') {
        return Ok(segment.to_string());
    }
    let bytes = segment.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            out.push(bytes[index]);
            index += 1;
            continue;
        }
        let hex = bytes
            .get(index + 1..index + 3)
            .ok_or_else(|| format!("route: truncated percent-escape in `{segment}`"))?;
        out.push(byte(hex, segment)?);
        index += 3;
    }
    String::from_utf8(out)
        .map_err(|_| format!("route: percent-escape in `{segment}` is not valid UTF-8"))
}

/// Convert two hex digits into a byte.
fn byte(hex: &[u8], segment: &str) -> Result<u8, String> {
    let text = std::str::from_utf8(hex)
        .map_err(|_| format!("route: invalid percent-escape in `{segment}`"))?;
    u8::from_str_radix(text, 16)
        .map_err(|_| format!("route: `%{text}` in `{segment}` is not a hex escape"))
}
