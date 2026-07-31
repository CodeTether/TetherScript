//! Unpadded base64url codec for CSRF token segments.
//!
//! # Why this is not the jwt group's codec
//!
//! `jwt::jwt_base64url::encode` is byte-identical in behaviour, but LSP hover
//! confirms it is `pub(super)` to the `jwt` group, so it is not reachable from
//! `csrf`. Widening it means editing another owner's file, which this task
//! forbids, so the codec is repeated here.
//!
//! `crate::system::base64_encode_bytes` is also unsuitable: it emits the standard
//! `+`/`/` alphabet with `=` padding, and a CSRF token travels in a URL query
//! parameter where all three characters need escaping.
//!
//! Consolidating the two copies is follow-up work for whoever owns both files.

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
/// Encoded text, never containing `=`, `+`, or `/`, so it is safe unescaped in a
/// URL query parameter.
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
