//! SHA-512 (FIPS 180-4), raw 64-byte digest plus lowercase hex.
//!
//! Unlike SHA-1, SHA-512 has no known collision or preimage weakness and is the
//! right default for new integrity and signature work in this tree.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::hash::sha512::{sha512, sha512_hex};
//!
//! // FIPS 180-4 §D.1, one-block "abc". Split for line width only.
//! let expected = concat!(
//!     "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a",
//!     "2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
//! );
//! assert_eq!(sha512_hex(b"abc"), expected);
//! assert_eq!(sha512(b"abc").len(), 64);
//! ```

use crate::hash::sha512_core::digest;
use crate::hash::sha512_iv::IV512;
use crate::system::hex_encode;

/// SHA-512 block size in bytes. HMAC-SHA-512 keys are padded to this width.
pub const BLOCK: usize = crate::hash::sha512_core::BLOCK;

/// Compute the raw 64-byte SHA-512 digest of `input`.
///
/// # Arguments
///
/// * `input` — Message bytes; any length, including empty.
///
/// # Returns
///
/// The 64-byte digest.
///
/// # Examples
///
/// ```rust
/// use tetherscript::hash::sha512::sha512;
///
/// // First byte of the FIPS 180-4 empty-message digest cf83e135...
/// assert_eq!(sha512(b"")[0], 0xcf);
/// ```
pub fn sha512(input: &[u8]) -> [u8; 64] {
    digest(IV512, input)
}

/// Compute the lowercase hex SHA-512 digest of `input`.
///
/// # Arguments
///
/// * `input` — Message bytes.
///
/// # Returns
///
/// A 128-character lowercase hex string.
///
/// # Examples
///
/// ```rust
/// use tetherscript::hash::sha512::sha512_hex;
///
/// assert!(sha512_hex(b"").starts_with("cf83e1357eefb8bd"));
/// assert_eq!(sha512_hex(b"").len(), 128);
/// ```
pub fn sha512_hex(input: &[u8]) -> String {
    hex_encode(&sha512(input))
}
