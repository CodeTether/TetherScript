//! # Modular exponentiation by square-and-multiply
//!
//! [`BigUint::modpow`] computes `base^exponent mod modulus`, the RSA primitive
//! itself: verification is `signature^e mod n`.
//!
//! ## Why a naive repeated-multiply loop cannot work
//!
//! The obvious implementation multiplies by `base` once per unit of the
//! exponent, so it performs `exponent` iterations. RSA's public exponent is
//! typically 65537 — already 65537 multiplications — and a *private* exponent
//! for a 2048-bit modulus is a ~2048-bit number, meaning on the order of
//! `2^2048` iterations. That is roughly `10^616`, vastly more than the number
//! of atoms in the observable universe; at a billion multiplications per second
//! the loop outlives the heat death of the universe. It does not terminate in
//! any practical sense, and no constant-factor optimization of the multiply
//! rescues it.
//!
//! Square-and-multiply is `O(bit_len(exponent))` instead of `O(exponent)`:
//! ~2048 squarings and at most ~2048 multiplies for the same 2048-bit exponent.
//! It exploits `x^(2k) = (x^k)^2` and `x^(2k+1) = (x^k)^2 · x`, walking the
//! exponent's bits from the most significant down: square the accumulator every
//! step, and multiply in the base when the current bit is set.
//!
//! Every step reduces modulo `modulus` (see [`BigUint::mulmod`]), so intermediates
//! stay at most `2n` limbs wide instead of growing without bound.
//!
//! ## Not constant time
//!
//! The multiply is skipped on a zero exponent bit and [`BigUint::divmod`]
//! iterates over significant bits, so the running time leaks the exponent's
//! Hamming weight. That is acceptable for RSA *verification*, where the exponent
//! is the public `e`. It is **not** safe for a secret exponent; a blinded or
//! constant-time ladder would be required first.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUint;
//!
//! let base = BigUint::from_u64(4);
//! let exponent = BigUint::from_u64(13);
//! let modulus = BigUint::from_u64(497);
//! assert_eq!(base.modpow(&exponent, &modulus).unwrap(), BigUint::from_u64(445));
//! ```

use super::error::BigUintError;
use super::limbs::BigUint;

impl BigUint {
    /// Compute `self^exponent mod modulus` by square-and-multiply.
    ///
    /// # Arguments
    ///
    /// * `exponent` — any width; `0` yields `1 mod modulus`. Cost is linear in
    ///   its bit length, not its value. See the module docs.
    /// * `modulus` — must be nonzero. Need not be prime or odd. `self` is
    ///   reduced first, so it may exceed the modulus.
    ///
    /// # Returns
    ///
    /// The normalized result, strictly less than `modulus`. With `modulus == 1`
    /// the result is `0`, including for a zero exponent, which falls out of
    /// reducing the initial accumulator rather than being special-cased.
    ///
    /// # Errors
    ///
    /// [`BigUintError::DivideByZero`] when `modulus` is zero, propagated from
    /// [`BigUint::divmod`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::{BigUint, BigUintError};
    ///
    /// let two = BigUint::from_u64(2);
    /// let thousand = BigUint::from_u64(1_000);
    /// // 2^10 mod 1000 == 24
    /// assert_eq!(
    ///     two.modpow(&BigUint::from_u64(10), &thousand).unwrap(),
    ///     BigUint::from_u64(24)
    /// );
    /// // x^0 == 1 and x^1 == x mod m
    /// assert!(two.modpow(&BigUint::zero(), &thousand).unwrap().is_one());
    /// assert_eq!(two.modpow(&BigUint::from_u64(1), &thousand).unwrap(), two);
    /// assert_eq!(
    ///     two.modpow(&BigUint::from_u64(3), &BigUint::zero()),
    ///     Err(BigUintError::DivideByZero)
    /// );
    /// ```
    pub fn modpow(&self, exponent: &Self, modulus: &Self) -> Result<Self, BigUintError> {
        let mut result = Self::from_u64(1).rem(modulus)?;
        let base = self.rem(modulus)?;
        for index in (0..exponent.bit_len()).rev() {
            result = result.mulmod(&result, modulus)?;
            if exponent.bit(index) {
                result = result.mulmod(&base, modulus)?;
            }
        }
        Ok(result)
    }
}
