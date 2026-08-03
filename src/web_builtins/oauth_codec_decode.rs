//! Unpadded base64url decoder for OAuth state segments.
//!
//! The counterpart to [`super::encode`]. Needed because a state token's payload
//! segment must be read back to recover the expiry and the return path, and because
//! the embedded return path is itself base64url so it may contain the `.` used as
//! the payload field separator.
//!
//! Padding is not accepted: nothing in this group ever emits `=`, so a `=` in the
//! input means the value was not produced here, and silently tolerating it would let
//! two different encodings of the same token both verify.

/// Map one base64url character to its 6-bit value.
///
/// # Returns
///
/// `None` for anything outside the URL-safe alphabet, including `=`, `+`, and `/`.
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

/// Decode unpadded base64url text.
///
/// # Arguments
///
/// * `label` — Name of the segment, used verbatim in error messages so the caller
///   learns *which* part of the token was malformed.
/// * `input` — Unpadded base64url text.
///
/// # Returns
///
/// The decoded bytes.
///
/// # Errors
///
/// Returns `Err` naming `label` when the length is impossible for unpadded base64url
/// (`len % 4 == 1`) or when a character falls outside the alphabet; the offending
/// character and its position are reported.
pub(crate) fn decode(label: &str, input: &str) -> Result<Vec<u8>, String> {
    if input.len() % 4 == 1 {
        return Err(format!("oauth: {label} is not valid unpadded base64url"));
    }
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0u32);
    for (position, byte) in input.bytes().enumerate() {
        let value = sextet(byte).ok_or_else(|| {
            let shown = byte as char;
            format!("oauth: {label} has invalid character `{shown}` at position {position}")
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
