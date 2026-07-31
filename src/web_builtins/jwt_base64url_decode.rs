//! Strict unpadded base64url decoding for JWS segments.
//!
//! Separate from the encoder so each file keeps one direction of the codec.

/// Map one base64url character to its 6-bit value.
///
/// `+` and `/` are rejected: a JWS segment must use the URL-safe alphabet, and
/// silently accepting the standard one would let a non-conforming token through.
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
/// * `label` — Segment name used in error text, such as `header` or `payload`.
/// * `input` — Encoded segment.
///
/// # Returns
///
/// The decoded bytes.
///
/// # Errors
///
/// Returns a named error when the segment contains `=` padding, a character
/// outside the URL-safe alphabet, or a length that no byte string can produce
/// (a single leftover character encodes only 6 bits).
pub(super) fn decode(label: &str, input: &str) -> Result<Vec<u8>, String> {
    let mut sextets = Vec::with_capacity(input.len());
    for byte in input.bytes() {
        match sextet(byte) {
            Some(value) => sextets.push(value),
            None if byte == b'=' => {
                return Err(format!(
                    "jwt: {label} is padded; base64url must be unpadded"
                ));
            }
            None => {
                return Err(format!(
                    "jwt: {label} has invalid base64url character `{}`",
                    byte as char
                ));
            }
        }
    }
    if sextets.len() % 4 == 1 {
        return Err(format!("jwt: {label} has a truncated base64url group"));
    }

    let mut out = Vec::with_capacity(sextets.len() / 4 * 3);
    for block in sextets.chunks(4) {
        let a = block[0];
        let b = block.get(1).copied().unwrap_or(0);
        let c = block.get(2).copied().unwrap_or(0);
        let d = block.get(3).copied().unwrap_or(0);
        out.push((a << 2) | (b >> 4));
        if block.len() > 2 {
            out.push((b << 4) | (c >> 2));
        }
        if block.len() > 3 {
            out.push((c << 6) | d);
        }
    }
    Ok(out)
}
