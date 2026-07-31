//! Strict unpadded base64url decoding for CSRF token segments.
//!
//! Separate from the encoder so each file owns one direction of the codec.

/// Map one base64url character to its 6-bit value.
///
/// `+`, `/`, and `=` are rejected. A token carrying them was not produced by
/// [`super::csrf_base64url::encode`], so tolerating them would accept a
/// malformed token as if it were well-formed.
fn sextet(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'-' => Some(62),
        b'_' => Some(63),
        _ => None,
    }
}

/// Decode unpadded base64url.
///
/// # Arguments
///
/// * `label` — Segment name used in error text, such as `payload`.
/// * `input` — Encoded segment.
///
/// # Returns
///
/// The decoded bytes.
///
/// # Errors
///
/// Returns an error naming `label`, and the offending character and its position,
/// when a character is outside the URL-safe alphabet or the length is invalid.
pub(super) fn decode(label: &str, input: &str) -> Result<Vec<u8>, String> {
    // A single leftover sextet cannot form a byte, so that length is impossible.
    if input.len() % 4 == 1 {
        return Err(format!(
            "csrf: {label} segment has an invalid unpadded base64url length"
        ));
    }
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    for (position, byte) in input.bytes().enumerate() {
        let value = sextet(byte).ok_or_else(|| {
            format!(
                "csrf: {label} segment has invalid base64url character `{}` at position {position}",
                byte as char
            )
        })?;
        acc = (acc << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Ok(out)
}
