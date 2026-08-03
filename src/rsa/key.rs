//! # The RSA public key type
//!
//! One responsibility: hold a validated RSA public key `(n, e)` and expose the
//! three facts a verifier needs — the modulus, the exponent, and the modulus
//! octet length `k`.
//!
//! ## Verification only
//!
//! This module and its siblings implement **signature verification and nothing
//! else**. There is deliberately no key generation, no private key type, no CRT
//! parameters, no decryption, and no signing. A private-key operation needs
//! blinding, constant-time modular inversion, and side-channel-resistant
//! exponentiation that the generic [`BigUint`] here does not provide, so
//! offering one would be actively dangerous. Public-key exponentiation operates
//! only on data an attacker already has, which is why plain square-and-multiply
//! is acceptable here.
//!
//! ## Validation happens at construction
//!
//! [`RsaPublicKey::new`] runs `super::key_check::admit`, so an
//! `RsaPublicKey` in hand is always at least 2048 bits, odd, and carries an
//! exponent of 2 or more. Callers never repeat those checks.
//!
//! # Examples
//!
//! ```rust,no_run
//! use tetherscript::bigint::BigUint;
//! use tetherscript::rsa::RsaPublicKey;
//!
//! // `n_bytes` and `e_bytes` come from a JWKS entry's `n` and `e` members,
//! // already base64url-decoded by `src/web_builtins/jwks_key.rs`.
//! let n_bytes: Vec<u8> = Vec::new();
//! let e_bytes: Vec<u8> = Vec::new();
//! let key = RsaPublicKey::new(
//!     BigUint::from_be_bytes(&n_bytes),
//!     BigUint::from_be_bytes(&e_bytes),
//! )
//! .expect("JWKS key material should be a usable RSA public key");
//! assert_eq!(key.modulus_bytes(), 256);
//! ```

use crate::bigint::BigUint;

use super::error::RsaError;
use super::key_check::admit;

/// A validated RSA public key. Verification only; see the module documentation
/// in `src/rsa/key.rs`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsaPublicKey {
    /// Validated modulus `n`. `pub(super)` so the sibling accessor and verify
    /// modules can read it; external code goes through
    /// [`RsaPublicKey::modulus`].
    pub(super) modulus: BigUint,
    /// Validated public exponent `e`, always at least 2.
    pub(super) exponent: BigUint,
}

impl RsaPublicKey {
    /// Build a key, rejecting unusable material.
    ///
    /// # Arguments
    ///
    /// * `modulus` — RSA modulus `n`.
    /// * `exponent` — public exponent `e`.
    ///
    /// # Returns
    ///
    /// The validated key.
    ///
    /// # Errors
    ///
    /// [`RsaError::ModulusTooSmall`] under 2048 bits, [`RsaError::ModulusEven`]
    /// for an even modulus, [`RsaError::ExponentTooSmall`] for `e` of 0 or 1.
    /// Each refusal blocks a concrete attack: a factorable modulus, a modulus
    /// that is not a product of odd primes, and an exponent under which every
    /// signature recovers the same block.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    /// use tetherscript::rsa::{RsaError, RsaPublicKey};
    ///
    /// let small = BigUint::from_u64(0xffff_ffff);
    /// let err = RsaPublicKey::new(small, BigUint::from_u64(65_537)).unwrap_err();
    /// assert_eq!(err, RsaError::ModulusTooSmall { bytes: 4 });
    /// ```
    pub fn new(modulus: BigUint, exponent: BigUint) -> Result<Self, RsaError> {
        admit(&modulus, &exponent)?;
        Ok(Self { modulus, exponent })
    }
}
