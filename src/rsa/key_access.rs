//! # Read-only accessors on [`RsaPublicKey`]
//!
//! One responsibility: expose the validated key's components without letting a
//! caller mutate them. Mutation after construction would bypass the admission
//! checks in `super::key_check`, so the fields stay private and are only read
//! through this file.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUint;
//! use tetherscript::rsa::RsaPublicKey;
//!
//! // Smallest admissible modulus: 2^2047 + 1, which is 256 bytes and odd.
//! let mut n = vec![0u8; 256];
//! n[0] = 0x80;
//! n[255] = 0x01;
//! let key = RsaPublicKey::new(
//!     BigUint::from_be_bytes(&n),
//!     BigUint::from_u64(65_537),
//! )
//! .unwrap();
//! assert_eq!(key.modulus_bytes(), 256);
//! assert_eq!(key.modulus_bits(), 2048);
//! assert_eq!(key.exponent(), &BigUint::from_u64(65_537));
//! ```

use crate::bigint::BigUint;

use super::key::RsaPublicKey;

impl RsaPublicKey {
    /// Borrow the modulus `n`.
    ///
    /// # Returns
    ///
    /// A reference to the validated modulus.
    pub fn modulus(&self) -> &BigUint {
        &self.modulus
    }

    /// Borrow the public exponent `e`.
    ///
    /// # Returns
    ///
    /// A reference to the validated exponent, always at least 2.
    pub fn exponent(&self) -> &BigUint {
        &self.exponent
    }

    /// The modulus length `k` in octets, which every signature must match.
    ///
    /// # Returns
    ///
    /// `ceil(modulus_bits / 8)`, at least 256.
    pub fn modulus_bytes(&self) -> usize {
        self.modulus.byte_len()
    }

    /// The modulus length in bits, counted from the most significant set bit.
    ///
    /// # Returns
    ///
    /// The significant bit length, at least 2041 (256 octets) in practice.
    pub fn modulus_bits(&self) -> usize {
        self.modulus.bit_len()
    }
}
