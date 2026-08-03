//! # Building a key from JWKS octet strings
//!
//! One responsibility: adapt the big-endian `n` and `e` octet strings that
//! `src/web_builtins/jwks_key.rs` produces into a validated
//! [`RsaPublicKey`]. That module hands out a map whose `modulus`/`n` and
//! `exponent`/`e` entries are raw base64url-decoded bytes, which is exactly the
//! shape [`BigUint::from_be_bytes`] consumes.
//!
//! ## Why this is separate from [`RsaPublicKey::new`]
//!
//! `new` is the arithmetic-typed constructor and knows nothing about octet
//! strings. Keeping the byte adapter in its own file means the JWKS integration
//! point is one obvious place, and the validation rules stay stated exactly once
//! in `super::key_check`.
//!
//! ## Leading zeros
//!
//! RFC 7518 section 6.3.1.1 says the base64url payload of `n` must be the
//! unsigned big-endian integer with no leading zero octets. This constructor
//! measures size from [`BigUint::byte_len`], the *significant* length, so a
//! non-conforming issuer that pads `n` with a leading zero cannot inflate its
//! apparent modulus size past the 2048-bit floor.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::rsa::{RsaError, RsaPublicKey};
//!
//! // 128 octets is 1024-bit: refused even though it is well-formed.
//! let mut weak = vec![0u8; 128];
//! weak[0] = 0xc0;
//! weak[127] = 0x01;
//! assert_eq!(
//!     RsaPublicKey::from_be_bytes(&weak, &[0x01, 0x00, 0x01]).unwrap_err(),
//!     RsaError::ModulusTooSmall { bytes: 128 }
//! );
//! ```

use crate::bigint::BigUint;

use super::error::RsaError;
use super::key::RsaPublicKey;

impl RsaPublicKey {
    /// Build a key from big-endian modulus and exponent octet strings.
    ///
    /// # Arguments
    ///
    /// * `modulus` — big-endian `n`, as decoded from a JWK `n` member.
    /// * `exponent` — big-endian `e`, typically `[0x01, 0x00, 0x01]` for 65537.
    ///
    /// # Returns
    ///
    /// The validated key.
    ///
    /// # Errors
    ///
    /// The same three admission errors as [`RsaPublicKey::new`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::rsa::RsaPublicKey;
    ///
    /// // 2^2047 + 1 is 256 octets and odd, the smallest admissible modulus.
    /// let mut n = vec![0u8; 256];
    /// n[0] = 0x80;
    /// n[255] = 0x01;
    /// let key = RsaPublicKey::from_be_bytes(&n, &[0x01, 0x00, 0x01]).unwrap();
    /// assert_eq!(key.modulus_bytes(), 256);
    /// ```
    pub fn from_be_bytes(modulus: &[u8], exponent: &[u8]) -> Result<Self, RsaError> {
        Self::new(
            BigUint::from_be_bytes(modulus),
            BigUint::from_be_bytes(exponent),
        )
    }
}
