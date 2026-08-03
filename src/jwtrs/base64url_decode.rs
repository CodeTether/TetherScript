//! Strict unpadded base64url decoding for JWS segments.
//!
//! One responsibility: the decode direction of the codec, and only the decode
//! direction. Separate from [`crate::jwtrs::base64url`] so the security-relevant
//! half is auditable on its own.
//!
//! # Security: rejection, not tolerance
//!
//! A lenient decoder gives one token several accepted encodings, which breaks
//! every downstream use of the compact serialization as a unique string — replay
//! caches, revocation lists, audit logs, rate-limit keys. So `=` padding, the
//! standard `+`/`/` alphabet, whitespace, and a dangling single character are all
//! **refused** rather than repaired.
//!
//! Rejecting `+` and `/` specifically matters: no conforming JWS signer emits
//! them, so their presence means the token came from something that is not a JWS
//! signer. Translating them to `-`/`_` and continuing would accept that input and
//! also make two distinct strings decode to the same bytes.

/// Map one base64url character to its 6-bit value.
///
/// `+`, `/`, and `=` fall through to `None`: a JWS segment must use the URL-safe
/// alphabet unpadded, and silently accepting the standard one would admit tokens
/// no conforming signer produces.
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
/// * `label` — Segment name used in the error text, such as `header`.
/// * `input` — The encoded segment.
///
/// # Returns
///
/// The decoded bytes.
///
/// # Errors
///
/// Returns a message naming the offending byte when the segment contains `=`
/// padding, a character outside the URL-safe alphabet, or a length no byte string
/// can produce — one leftover character carries only 6 bits, which is less than a
/// byte and so cannot be the tail of any encoding.
///
/// # Panics
///
/// Does not panic.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwtrs::base64url_decode::decode;
///
/// assert_eq!(decode("payload", "eyJhIjoxfQ").unwrap().as_slice(), br#"{"a":1}"#.as_slice());
/// // Standard base64 is refused, not translated.
/// assert!(decode("payload", "ab+d").is_err());
/// assert!(decode("payload", "abcd=").is_err());
/// // A single leftover character is not a valid tail.
/// assert!(decode("payload", "abcde").is_err());
/// ```
pub fn decode(label: &str, input: &str) -> Result<Vec<u8>, String> {
    let mut sextets = Vec::with_capacity(input.len());
    for byte in input.bytes() {
        let Some(value) = sextet(byte) else {
            return Err(match byte {
                b'=' => format!("{label}: `=` padding is not allowed in a JWS segment"),
                b'+' | b'/' => format!(
                    "{label}: `{}` is standard base64; JWS requires the URL-safe alphabet",
                    byte as char
                ),
                _ => format!("{label}: byte 0x{byte:02x} is not a base64url character"),
            });
        };
        sextets.push(value);
    }
    if sextets.len() % 4 == 1 {
        return Err(format!(
            "{label}: length {} leaves a 6-bit remainder, which encodes no byte",
            sextets.len()
        ));
    }
    Ok(pack(&sextets))
}

/// Repack 6-bit groups into bytes, discarding the sub-byte tail.
///
/// The tail bits of a truncated final group are padding by construction, so
/// dropping them is the inverse of the encoder rather than a loss.
fn pack(sextets: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(sextets.len() * 3 / 4);
    let mut accumulator: u32 = 0;
    let mut bits = 0;
    for value in sextets {
        accumulator = (accumulator << 6) | u32::from(*value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((accumulator >> bits) & 0xff) as u8);
        }
    }
    out
}
