//! Verifier generation and S256 challenge derivation.
//!
//! The rules and the reasoning live in [`super`]; this file is the mechanics. Split out
//! so the rationale and the arithmetic can each be read without the other, and to stay
//! inside the 50-line limit.

use super::super::codec::encode;
use super::super::entropy::bytes;
use super::{unreserved, MAX_VERIFIER, MIN_VERIFIER};
use crate::system::sha256;

/// Mint a fresh 43-character code verifier.
///
/// 32 bytes of entropy encode to exactly 43 unpadded base64url characters, which is both
/// the RFC minimum length and the 256 bits of entropy RFC 7636 §7.1 asks for. base64url
/// output is a subset of the unreserved set, so the result never needs percent-encoding.
///
/// # Returns
///
/// A 43-character verifier drawn from fresh OS entropy.
pub(crate) fn generate() -> String {
    encode(&bytes(32))
}

/// Derive the S256 code challenge for `verifier`.
///
/// The challenge is `BASE64URL(SHA256(ASCII(verifier)))` per RFC 7636 §4.2 — the digest
/// is taken over the verifier's *ASCII bytes*, not over any decoding of it, which is why
/// the bytes are hashed directly.
///
/// # Arguments
///
/// * `verifier` — A code verifier, validated here rather than trusted.
///
/// # Returns
///
/// The 43-character unpadded base64url challenge.
///
/// # Errors
///
/// Returns `Err` naming the actual length when `verifier` is shorter than 43 or longer
/// than 128 characters, and naming the offending character and its position when it falls
/// outside the unreserved set.
pub(crate) fn challenge(verifier: &str) -> Result<String, String> {
    validate(verifier)?;
    Ok(encode(&sha256(verifier.as_bytes())))
}

/// Reject a verifier that violates RFC 7636 §4.1.
fn validate(verifier: &str) -> Result<(), String> {
    let len = verifier.len();
    if !(MIN_VERIFIER..=MAX_VERIFIER).contains(&len) {
        return Err(format!(
            "oauth_pkce_challenge: code_verifier must be {MIN_VERIFIER}-{MAX_VERIFIER} characters, got {len}"
        ));
    }
    match verifier.bytes().position(|byte| !unreserved(byte)) {
        Some(position) => Err(format!(
            "oauth_pkce_challenge: code_verifier has disallowed character `{}` at position {position}; only A-Z a-z 0-9 - . _ ~ are permitted",
            verifier.as_bytes()[position] as char
        )),
        None => Ok(()),
    }
}
