//! Unpadded base64url encoding for JWS segments (RFC 4648 §5, RFC 7515 §2).
//!
//! One responsibility: the encode direction of the byte-level codec. No JSON, no
//! claims, no policy. The strict decoder — the security-relevant direction — is
//! [`crate::jwtrs::base64url_decode`], so each file owns one direction.
//!
//! # Why this is not `crate::system::base64_*` and not the HS256 group's copy
//!
//! `crate::system` implements *standard* base64: the `+`/`/` alphabet with `=`
//! padding. JWS requires the URL-safe alphabet with padding stripped, so the
//! standard codec is the wrong grammar entirely.
//!
//! The HS256 group already has a correct strict codec in
//! `src/web_builtins/jwt_base64url.rs`, but it is `pub(super)`-scoped to
//! `web_builtins` and therefore structurally unreachable from `crate::jwtrs`.
//! Reaching it would mean widening another group's visibility, which this task is
//! not allowed to do; `src/jwks/base64url.rs` exists for the same reason. So this
//! is a third instance of a ~20-line grammar rather than a shared helper — a
//! deliberate, bounded duplication, recorded here so a future integrator can
//! collapse all three in one focused change.

/// URL-safe alphabet: the standard table with `+` and `/` replaced.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode bytes as unpadded base64url.
///
/// Present so tests and doc examples can build fixture tokens without a second
/// crate. Nothing in the validation path calls it.
///
/// # Arguments
///
/// * `bytes` — Raw input.
///
/// # Returns
///
/// The encoded text, never containing `=`, `+`, or `/`.
///
/// # Panics
///
/// Does not panic.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwtrs::base64url::encode;
/// use tetherscript::jwtrs::base64url_decode::decode;
///
/// let text = encode(br#"{"alg":"RS256"}"#);
/// assert!(!text.contains('='));
/// assert_eq!(decode("header", &text).unwrap().as_slice(), br#"{"alg":"RS256"}"#.as_slice());
/// ```
pub fn encode(bytes: &[u8]) -> String {
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
