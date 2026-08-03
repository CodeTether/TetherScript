//! SHA-384 (FIPS 180-4): the SHA-512 compression function run from the SHA-384
//! initial hash value, then truncated to 48 bytes.
//!
//! # Not a truncated SHA-512
//!
//! Truncating a SHA-512 digest to 48 bytes gives a *different* value, because
//! SHA-384 starts the chain from a distinct IV (square roots of primes 23..53
//! rather than 2..19). `tests/hash_sha512.rs` asserts the inequality so nobody
//! "optimizes" this into a slice of `sha512`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::hash::sha384::{sha384, sha384_hex};
//!
//! // FIPS 180-4 §D.3, one-block "abc".
//! let expected = concat!(
//!     "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed",
//!     "8086072ba1e7cc2358baeca134c825a7",
//! );
//! assert_eq!(sha384_hex(b"abc"), expected);
//! assert_eq!(sha384(b"abc").len(), 48);
//! ```

use crate::hash::sha512_core::digest;
use crate::hash::sha512_iv::IV384;
use crate::system::hex_encode;

/// SHA-384 block size in bytes; identical to SHA-512's.
pub const BLOCK: usize = crate::hash::sha512_core::BLOCK;

/// Compute the raw 48-byte SHA-384 digest of `input`.
///
/// # Arguments
///
/// * `input` — Message bytes; any length, including empty.
///
/// # Returns
///
/// The 48-byte digest.
///
/// # Examples
///
/// ```rust
/// use tetherscript::hash::sha384::sha384;
///
/// // First byte of the empty-message digest 38b060a7...
/// assert_eq!(sha384(b"")[0], 0x38);
/// ```
pub fn sha384(input: &[u8]) -> [u8; 48] {
    let full = digest(IV384, input);
    let mut out = [0u8; 48];
    out.copy_from_slice(&full[..48]);
    out
}

/// Compute the lowercase hex SHA-384 digest of `input`.
///
/// # Arguments
///
/// * `input` — Message bytes.
///
/// # Returns
///
/// A 96-character lowercase hex string.
///
/// # Examples
///
/// ```rust
/// use tetherscript::hash::sha384::sha384_hex;
///
/// assert!(sha384_hex(b"").starts_with("38b060a751ac9638"));
/// assert_eq!(sha384_hex(b"").len(), 96);
/// ```
pub fn sha384_hex(input: &[u8]) -> String {
    hex_encode(&sha384(input))
}
