//! Unpadded base64url codec (RFC 4648 §5).
//!
//! # Why this is not `crate::system::base64_encode_bytes`
//!
//! Those helpers implement standard base64: the `+`/`/` alphabet with `=`
//! padding. JWS compact serialization requires the URL-safe alphabet (`-`/`_`)
//! with padding stripped, and it requires decoders to *reject* padding rather
//! than tolerate it — a `=` in a segment means the token was not produced by a
//! conforming signer. Translating the standard output would still need a
//! separate strict decoder, so the codec is written directly against the JWS
//! rules instead.

/// URL-safe alphabet: the standard table with `+` and `/` replaced.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode bytes as unpadded base64url.
///
/// # Arguments
///
/// * `bytes` — Raw input.
///
/// # Returns
///
/// The encoded text, never containing `=`, `+`, or `/`.
pub(super) fn encode(bytes: &[u8]) -> String {
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
