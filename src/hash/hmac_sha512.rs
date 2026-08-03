//! HMAC-SHA-512 (RFC 2104 / RFC 4231).
//!
//! # The block size is 128, not 64
//!
//! HMAC's block size is the *hash's internal block size*, not its digest size.
//! SHA-512 compresses 128-byte blocks, so `BLOCK` here is 128 — copied from
//! [`crate::hash::sha512::BLOCK`] rather than written as a literal so the two
//! cannot drift. Reusing SHA-1's 64 would not fail loudly: keys of 1..64 bytes
//! would still produce correct MACs, while keys of 65..128 bytes would be
//! wrongly hashed down to 64 bytes instead of being zero-padded. The result is a
//! self-consistent but non-interoperable MAC — exactly the bug RFC 4231 test
//! case 6 (a 131-byte key) and a 65-byte-key check in `tests/hash_hmac.rs`
//! are there to catch.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::hash::hmac_sha512::hmac_sha512;
//!
//! // RFC 4231 test case 2, first two bytes of 164b7a7b...
//! let mac = hmac_sha512(b"Jefe", b"what do ya want for nothing?");
//! assert_eq!(&mac[..2], &[0x16, 0x4b]);
//! ```

use crate::hash::sha512::{sha512, BLOCK};
use crate::system::hex_encode;

/// HMAC-SHA-512 per RFC 2104, with the 128-byte SHA-512 block.
///
/// # Arguments
///
/// * `key` — Secret key. Keys longer than 128 bytes are hashed first; shorter
///   keys are zero-padded to 128 bytes.
/// * `message` — Bytes to authenticate.
///
/// # Returns
///
/// The 64-byte message authentication code.
pub fn hmac_sha512(key: &[u8], message: &[u8]) -> [u8; 64] {
    let mut block = [0u8; BLOCK];
    if key.len() > BLOCK {
        block[..64].copy_from_slice(&sha512(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let mut inner = Vec::with_capacity(BLOCK + message.len());
    let mut outer = Vec::with_capacity(BLOCK + 64);
    for byte in block.iter() {
        inner.push(byte ^ 0x36);
        outer.push(byte ^ 0x5c);
    }
    inner.extend_from_slice(message);
    outer.extend_from_slice(&sha512(&inner));
    sha512(&outer)
}

/// Lowercase hex form of [`hmac_sha512`].
///
/// # Arguments
///
/// * `key` — Secret key.
/// * `message` — Bytes to authenticate.
///
/// # Returns
///
/// A 128-character lowercase hex string.
pub fn hmac_sha512_hex(key: &[u8], message: &[u8]) -> String {
    hex_encode(&hmac_sha512(key, message))
}
