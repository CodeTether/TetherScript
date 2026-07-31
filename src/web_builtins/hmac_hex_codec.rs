//! Lowercase hex encoding and strict decoding.
//!
//! Encoding delegates to [`crate::system::hex_encode`] so there is exactly one
//! hex table in the tree. Decoding lives here because `system` has no decoder.

use crate::system::hex_encode;

/// Encode bytes as lowercase hex.
///
/// # Arguments
///
/// * `bytes` — Input bytes.
///
/// # Returns
///
/// A hex string of exactly `2 * bytes.len()` characters.
pub(super) fn encode_hex(bytes: &[u8]) -> String {
    hex_encode(bytes)
}

/// Decode a hex string, rejecting odd lengths and non-hex characters.
///
/// # Arguments
///
/// * `input` — Hex text. Upper and lower case are both accepted.
///
/// # Returns
///
/// The decoded bytes.
///
/// # Errors
///
/// Returns an error when `input` has an odd length, or when a character is not a
/// hex digit. The message names the offending character and its position so the
/// caller can find it.
pub(super) fn decode_hex(input: &str) -> Result<Vec<u8>, String> {
    if !input.len().is_multiple_of(2) {
        return Err(format!(
            "hex_decode: odd length {}; hex needs two characters per byte",
            input.len()
        ));
    }
    let digits: Vec<char> = input.chars().collect();
    let mut out = Vec::with_capacity(digits.len() / 2);
    for (index, pair) in digits.chunks(2).enumerate() {
        let high = nibble(pair[0], index * 2)?;
        let low = nibble(pair[1], index * 2 + 1)?;
        out.push((high << 4) | low);
    }
    Ok(out)
}

/// Convert one hex digit, naming it when invalid.
fn nibble(digit: char, position: usize) -> Result<u8, String> {
    digit.to_digit(16).map(|value| value as u8).ok_or_else(|| {
        format!("hex_decode: invalid hex character `{digit}` at position {position}")
    })
}
