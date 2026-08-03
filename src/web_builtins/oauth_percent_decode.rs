//! Percent-decoding for callback query values.
//!
//! The counterpart to [`super::encode`]. `+` is decoded to a space, matching the
//! `application/x-www-form-urlencoded` rules a browser follows when it builds the
//! redirect it sends back, so an `error_description` containing spaces reads correctly
//! whichever form the provider chose.
//!
//! A truncated or invalid escape is an error rather than a silent pass-through.
//! Tolerating `%zz` means the value a script sees differs from the value the provider
//! sent, and that kind of divergence is how a decoding bug becomes a validation bypass.

/// Parse one hex digit, accepting either case.
fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Percent-decode a query component.
///
/// # Arguments
///
/// * `input` — Raw, still-encoded component.
/// * `label` — Name used in error messages so the caller learns which field failed.
///
/// # Returns
///
/// The decoded UTF-8 text.
///
/// # Errors
///
/// Returns `Err` naming `label` when an escape is truncated, when it is not two hex
/// digits, or when the decoded bytes are not valid UTF-8.
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

/// Read the two hex digits following a `%` at `index`.
fn escape_at(bytes: &[u8], index: usize, label: &str) -> Result<u8, String> {
    let pair = bytes
        .get(index + 1..index + 3)
        .ok_or_else(|| format!("{label}: truncated percent escape at position {index}"))?;
    match (nibble(pair[0]), nibble(pair[1])) {
        (Some(high), Some(low)) => Ok((high << 4) | low),
        _ => Err(format!(
            "{label}: invalid percent escape `%{}`",
            String::from_utf8_lossy(pair)
        )),
    }
}
