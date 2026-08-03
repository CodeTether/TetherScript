//! # Public-key admission checks
//!
//! One responsibility: decide whether a modulus/exponent pair is a *usable* RSA
//! public key. `super::key` calls this from its constructor so an
//! [`RsaPublicKey`](crate::rsa::RsaPublicKey) value can never exist in a
//! rejected state, and no caller has to remember to validate.
//!
//! ## The four refusals and why each one matters
//!
//! 1. **Modulus under 256 octets (2048 bits).** 1024-bit RSA is factorable by a
//!    well-resourced adversary. Refusing at construction means a weak JWKS entry
//!    cannot be silently trusted by a caller that only checks the return of
//!    `verify`. This mirrors the same floor enforced in
//!    `src/web_builtins/jwks_rsa.rs`, so the two layers agree.
//! 2. **Even modulus.** `n = p * q` for odd primes `p`, `q` is always odd. An
//!    even modulus is either corruption or an attacker-chosen value; with
//!    `n = 2^k` the map `s -> s^e mod n` is far from a permutation and forging
//!    becomes easy.
//! 3. **Exponent 0.** `s^0 mod n == 1` for every `s`, so *every* signature
//!    recovers the same encoded message. A key with `e = 0` verifies whatever an
//!    attacker wants, if the attacker can pick the digest.
//! 4. **Exponent 1.** `s^1 mod n == s`, so the "signature" is just the encoded
//!    message written out in the clear. Anyone can produce it without the
//!    private key.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUint;
//! use tetherscript::rsa::{RsaError, RsaPublicKey};
//!
//! // 2048-bit but even -> refused.
//! let mut n = vec![0u8; 256];
//! n[0] = 0x80;
//! let err = RsaPublicKey::new(BigUint::from_be_bytes(&n), BigUint::from_u64(65_537))
//!     .unwrap_err();
//! assert_eq!(err, RsaError::ModulusEven);
//! ```

use crate::bigint::BigUint;

use super::error::RsaError;

/// Minimum accepted modulus length in octets (2048 bits).
pub const MIN_MODULUS_BYTES: usize = 256;

/// Validate a candidate modulus and exponent.
///
/// # Arguments
///
/// * `modulus` — big-endian RSA modulus `n` as a [`BigUint`].
/// * `exponent` — public exponent `e` as a [`BigUint`].
///
/// # Returns
///
/// `Ok(())` when all four admission rules pass.
///
/// # Errors
///
/// [`RsaError::ModulusTooSmall`], [`RsaError::ModulusEven`], or
/// [`RsaError::ExponentTooSmall`], in that order of precedence.
pub(super) fn admit(modulus: &BigUint, exponent: &BigUint) -> Result<(), RsaError> {
    let bytes = modulus.byte_len();
    if bytes < MIN_MODULUS_BYTES {
        return Err(RsaError::ModulusTooSmall { bytes });
    }
    // `bit(0)` reads the least significant bit; a zero modulus is already
    // excluded by the size check above, so this is safe to read directly.
    if !modulus.bit(0) {
        return Err(RsaError::ModulusEven);
    }
    if exponent.is_zero() || exponent.is_one() {
        return Err(RsaError::ExponentTooSmall);
    }
    Ok(())
}
