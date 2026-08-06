//! Strict base64url decoding for JWK `n` and `e` members (RFC 4648 §5).
//!
//! One responsibility: validate the base64url alphabet and hand the bytes to the
//! crate's existing base64 decoder. No new bit-twiddling lives here.
//!
//! # Which decoder this reuses
//!
//! `crate::system::base64_decode_bytes` — the in-tree, dependency-free
//! standard-base64 decoder that already backs the `base64_decode` built-in and
//! the PostgreSQL SCRAM exchange. This file is only an *alphabet adapter*: it
//! rejects anything outside base64url, translates `-` to `+` and `_` to `/`, and
//! appends the `=` padding that decoder requires. Reusing it means the padding,
//! grouping, and bit-packing rules are the ones the rest of the crate is already
//! tested against.
//!
//! # Security: base64url is not base64
//!
//! A JWK `n` is specified as *unpadded base64url*. Decoding it with a standard
//! base64 decoder that silently tolerates both alphabets is the classic way to
//! end up with a subtly wrong modulus: `-` and `_` would decode to whatever the
//! standard table says (or be skipped as noise), producing a modulus that is
//! wrong in a handful of bits. Every signature then fails with no diagnostic,
//! because the arithmetic is correct and only the key is wrong. So `+`, `/`, and
//! `=` are *refused*, never translated — a document using them is malformed, and
//! saying so is more useful than guessing.

use crate::jwks::limits::MAX_FIELD_CHARS;
use crate::system::base64_decode_bytes;

/// Decode strict unpadded base64url into big-endian bytes.
///
/// # Arguments
///
/// * `label` — Locating name used in error text, such as `jwks: keys[0].n`.
/// * `input` — The encoded member text.
///
/// # Returns
///
/// The decoded bytes, in the same order as the encoded text — which for a JWK
/// `n` or `e` member means big-endian.
///
/// # Errors
///
/// Returns a named error when `input` is longer than
/// [`MAX_FIELD_CHARS`], carries `=`
/// padding, uses the standard `+`/`/` alphabet, contains any other non-alphabet
/// byte, or has length `4n + 1`, which encodes 6 leftover bits and so no byte
/// string can produce.
///
/// # Panics
///
/// Does not panic.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::base64url::decode;
///
/// // `-` and `_`, unpadded: the two bytes 0xFB 0xFF.
/// assert_eq!(decode("example", "-_8").unwrap(), vec![0xfb, 0xff]);
/// // The standard alphabet is refused rather than reinterpreted.
/// assert!(decode("example", "+/8").is_err());
/// assert!(decode("example", "-_8=").is_err());
/// ```
pub fn decode(label: &str, input: &str) -> Result<Vec<u8>, String> {
    if input.len() > MAX_FIELD_CHARS {
        return Err(format!(
            "{label}: {} encoded bytes exceeds the {MAX_FIELD_CHARS} byte field limit",
            input.len()
        ));
    }
    let mut standard = String::with_capacity(input.len() + 3);
    for byte in input.bytes() {
        standard.push(translate(label, byte)?);
    }
    pad(label, &mut standard)?;
    base64_decode_bytes(&standard).map_err(|error| format!("{label}: {error}"))
}

/// Map one base64url byte to its standard-base64 spelling.
fn translate(label: &str, byte: u8) -> Result<char, String> {
    match byte {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' => Ok(byte as char),
        b'-' => Ok('+'),
        b'_' => Ok('/'),
        b'=' => Err(format!("{label}: base64url must be unpadded, found `=`")),
        b'+' | b'/' => Err(format!(
            "{label}: `{}` is standard base64, not base64url",
            byte as char
        )),
        other => Err(format!("{label}: invalid base64url byte 0x{other:02x}")),
    }
}

/// Restore the `=` padding the reused standard decoder requires.
fn pad(label: &str, standard: &mut String) -> Result<(), String> {
    match standard.len() % 4 {
        0 => Ok(()),
        1 => Err(format!(
            "{label}: truncated base64url; a {}-character group is impossible",
            standard.len()
        )),
        remainder => {
            standard.push_str(&"=".repeat(4 - remainder));
            Ok(())
        }
    }
}
