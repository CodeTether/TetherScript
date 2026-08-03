//! Strict unpadded base64url alphabet validation (RFC 4648 §5).
//!
//! # Why a second decoder exists
//!
//! `web_builtins::jwt` already has one, but its helpers are `pub(super)` — that
//! is, private to the `jwt` group — and this task must not edit another group's
//! files. Duplicating ~40 lines is the cheaper of the two options, and the two
//! copies cannot drift in a way that matters because both are pinned by tests
//! that assert the same rejection set.
//!
//! # Security
//!
//! `+` and `/` are refused, not translated. A JWS segment and a JWK `n`/`e`
//! member are both specified as base64url; accepting the standard alphabet would
//! give one key two spellings, which defeats any cache or comparison keyed on the
//! encoded text. `=` padding is refused for the same reason.

use super::jwks_base64url_pack::pack;

/// Map one base64url character to its 6-bit value.
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

/// Decode strict unpadded base64url.
///
/// # Arguments
///
/// * `label` — Field name used in error text, such as `jwks: key[0].n`.
/// * `input` — Encoded text.
///
/// # Returns
///
/// The decoded bytes.
///
/// # Errors
///
/// Returns a named error when `input` carries `=` padding, uses the standard
/// `+`/`/` alphabet, contains any other non-alphabet byte, or has a length of
/// `4n + 1`, which encodes only 6 leftover bits and so no byte string can
/// produce.
///
/// # Examples
///
/// ```tether
/// println(jwt_header("eyJhbGciOiJSUzI1NiJ9.e30.AA").unwrap().alg)   // RS256
/// ```
pub(super) fn decode(label: &str, input: &str) -> Result<Vec<u8>, String> {
    let mut sextets = Vec::with_capacity(input.len());
    for byte in input.bytes() {
        match sextet(byte) {
            Some(value) => sextets.push(value),
            None => return Err(reject(label, byte)),
        }
    }
    if sextets.len() % 4 == 1 {
        return Err(format!("{label}: truncated base64url group"));
    }
    Ok(pack(&sextets))
}

/// Build the named error for one rejected byte.
fn reject(label: &str, byte: u8) -> String {
    match byte {
        b'=' => format!("{label}: base64url must be unpadded, found `=`"),
        b'+' | b'/' => format!(
            "{label}: `{}` is standard base64, not base64url",
            byte as char
        ),
        _ => format!("{label}: invalid base64url character `{}`", byte as char),
    }
}
