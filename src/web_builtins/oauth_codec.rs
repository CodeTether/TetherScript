//! Unpadded base64url codec for OAuth state segments and PKCE challenges.
//!
//! # Why a local copy
//!
//! Three byte-identical encoders already exist in the tree
//! (`csrf_base64url::encode`, `random_codec::base64_url`,
//! `jwt::jwt_base64url::encode`), but each is `pub(super)` to its own group and
//! widening one means editing a file this task must not touch. So the table is
//! repeated here rather than reached across an ownership boundary.
//!
//! [`crate::system::base64_encode_bytes`] is not a substitute: it emits the
//! standard `+`/`/` alphabet with `=` padding. RFC 7636 §4.2 defines a PKCE
//! `code_challenge` as base64url **without** padding, and all three of those
//! characters would additionally need escaping in a query parameter.
//!
//! Consolidating the copies is follow-up work for whoever owns all four files.

#[path = "oauth_codec_decode.rs"]
pub(crate) mod decode;

/// URL-safe alphabet: the standard table with `+` and `/` replaced by `-` and `_`.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode bytes as unpadded base64url.
///
/// # Arguments
///
/// * `bytes` — Raw input, typically a 32-byte SHA-256 digest or an HMAC tag.
///
/// # Returns
///
/// Encoded text containing no `=`, `+`, or `/`, so it is safe unescaped in a URL
/// query parameter. A 32-byte input yields 43 characters, the PKCE challenge
/// length.
pub(crate) fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        // Trailing groups are truncated rather than padded with `=`.
        if chunk.len() > 1 {
            out.push(ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(b2 & 0x3f) as usize] as char);
        }
    }
    out
}
