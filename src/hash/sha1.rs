//! SHA-1 (RFC 3174 / FIPS 180-4), raw digest plus lowercase hex.
//!
//! # Security warning
//!
//! **SHA-1 is collision-broken.** Practical chosen-prefix collisions exist
//! (SHAttered, 2017; `sha1collider`, 2019), so SHA-1 must not be used for new
//! signatures, certificates, or content-integrity checks. It is present in this
//! tree only for *protocol compatibility* with formats that hard-code it and
//! whose security does not rest on collision resistance: the RFC 6455 WebSocket
//! `Sec-WebSocket-Accept` transformation and git object identity. For anything
//! new, use [`crate::hash::sha512`] or the existing SHA-256.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::hash::sha1::{sha1, sha1_hex};
//!
//! assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
//! assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
//! assert_eq!(sha1(b"abc").len(), 20);
//! ```

use crate::hash::pad_block::padded64;
use crate::hash::sha1_block::compress;
use crate::system::hex_encode;

/// SHA-1 initial hash value, RFC 3174 §6.1.
const IV: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];

/// SHA-1 block size in bytes.
pub const BLOCK: usize = 64;

/// Compute the raw 20-byte SHA-1 digest of `input`.
///
/// # Arguments
///
/// * `input` — Message bytes; any length, including empty.
///
/// # Returns
///
/// The 20-byte digest.
///
/// # Examples
///
/// ```rust
/// use tetherscript::hash::sha1::sha1;
///
/// // RFC 3174 §7.3 test vector 1.
/// assert_eq!(sha1(b"abc")[0], 0xa9);
/// ```
pub fn sha1(input: &[u8]) -> [u8; 20] {
    let mut h = IV;
    for chunk in padded64(input).chunks_exact(BLOCK) {
        compress(&mut h, chunk);
    }
    let mut out = [0u8; 20];
    for (bytes, word) in out.chunks_exact_mut(4).zip(h) {
        bytes.copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Compute the lowercase hex SHA-1 digest of `input`.
///
/// # Arguments
///
/// * `input` — Message bytes.
///
/// # Returns
///
/// A 40-character lowercase hex string.
///
/// # Examples
///
/// ```rust
/// use tetherscript::hash::sha1::sha1_hex;
///
/// assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
/// ```
pub fn sha1_hex(input: &[u8]) -> String {
    hex_encode(&sha1(input))
}
