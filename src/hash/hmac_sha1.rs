//! HMAC-SHA-1 (RFC 2104), for protocol compatibility only.
//!
//! HMAC-SHA-1 is not broken by SHA-1's collision weakness — HMAC security rests
//! on PRF properties, not collision resistance — but new designs should still
//! prefer [`crate::hash::hmac_sha512`]. This exists for legacy protocols
//! (RFC 6238 TOTP, AWS SigV2, older SASL mechanisms) that hard-code it.
//!
//! Follows the same shape as `src/web_builtins/hmac_digest.rs`: fixed block
//! buffer, oversized key hashed first, `ipad = 0x36`, `opad = 0x5c`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::hash::hmac_sha1::hmac_sha1_hex;
//!
//! // RFC 2202 test case 2.
//! assert_eq!(
//!     hmac_sha1_hex(b"Jefe", b"what do ya want for nothing?"),
//!     "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79"
//! );
//! ```

use crate::hash::sha1::{BLOCK, sha1};
use crate::system::hex_encode;

/// HMAC-SHA-1 per RFC 2104.
///
/// # Arguments
///
/// * `key` — Secret key. Keys longer than the 64-byte SHA-1 block are hashed
///   first, per RFC 2104 §2; shorter keys are zero-padded to 64 bytes.
/// * `message` — Bytes to authenticate.
///
/// # Returns
///
/// The 20-byte message authentication code.
pub fn hmac_sha1(key: &[u8], message: &[u8]) -> [u8; 20] {
    let mut block = [0u8; BLOCK];
    if key.len() > BLOCK {
        block[..20].copy_from_slice(&sha1(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let mut inner = Vec::with_capacity(BLOCK + message.len());
    let mut outer = Vec::with_capacity(BLOCK + 20);
    for byte in block.iter() {
        inner.push(byte ^ 0x36);
        outer.push(byte ^ 0x5c);
    }
    inner.extend_from_slice(message);
    outer.extend_from_slice(&sha1(&inner));
    sha1(&outer)
}

/// Lowercase hex form of [`hmac_sha1`].
///
/// # Arguments
///
/// * `key` — Secret key.
/// * `message` — Bytes to authenticate.
///
/// # Returns
///
/// A 40-character lowercase hex string.
pub fn hmac_sha1_hex(key: &[u8], message: &[u8]) -> String {
    hex_encode(&hmac_sha1(key, message))
}
