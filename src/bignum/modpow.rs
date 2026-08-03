//! Modular exponentiation — the reason this module exists.
//!
//! [`Uint::mod_pow`] is right-to-left binary square-and-multiply: walk the
//! exponent from its least significant bit, squaring a running base each step
//! and folding it into the accumulator whenever the bit is set. That is
//! `O(bit_len(exponent))` modular multiplications — about 2048 squarings and up
//! to 2048 multiplies for a 2048-bit exponent — versus the impossible
//! `2^2048` of repeated multiplication.
//!
//! # Timing: this is NOT constant time
//!
//! Stated explicitly rather than left to inference. Two things vary with the
//! secret-independent inputs:
//!
//! 1. the multiply is skipped when an exponent bit is zero, so the running time
//!    leaks the exponent's Hamming weight; and
//! 2. the long division inside [`Uint::mul_mod`] has data-dependent branches
//!    (Knuth's add-back step, and the estimate correction loop).
//!
//! This is **defensible for the intended use**: RSA *signature verification*
//! exponentiates public data with the public exponent (typically 65537), so
//! there is no secret to leak. It would be **unsafe for RSA private-key
//! operations, decryption, or Diffie-Hellman**, where the exponent is secret. Do
//! not reach for this function there without adding a constant-time ladder.
//!
//! # Edge cases
//!
//! * `modulus == 1` — every residue is zero, so the result is zero even for a
//!   zero exponent. Handled before the loop, since `1 mod 1 == 0` must not be
//!   short-circuited to `1`.
//! * `exponent == 0` — the result is `1 mod modulus`.
//! * `base == 0` with a positive exponent — zero.
//! * `base >= modulus` — reduced up front, so oversized bases are fine.

use super::uint::Uint;

impl Uint {
    /// Computes `base^exponent mod modulus`.
    ///
    /// # Arguments
    ///
    /// * `base` — the base; may be larger than `modulus` and is reduced first.
    /// * `exponent` — the exponent; zero yields `1 mod modulus`.
    /// * `modulus` — a non-zero modulus. A modulus of one yields zero.
    ///
    /// # Returns
    ///
    /// The reduced power, always strictly less than `modulus`.
    ///
    /// # Panics
    ///
    /// Panics when `modulus` is zero, naming the base and exponent.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// // 3^4 mod 5 == 81 mod 5 == 1
    /// let got = Uint::mod_pow(&Uint::from_u64(3), &Uint::from_u64(4), &Uint::from_u64(5));
    /// assert_eq!(got, Uint::one());
    ///
    /// // Exponent zero, and modulus one.
    /// let m = Uint::from_u64(7);
    /// assert_eq!(Uint::mod_pow(&Uint::from_u64(5), &Uint::zero(), &m), Uint::one());
    /// assert!(Uint::mod_pow(&Uint::from_u64(5), &Uint::from_u64(3), &Uint::one()).is_zero());
    ///
    /// // Fermat: 2^(p-1) mod p == 1 for the prime p = 1_000_003.
    /// let p = Uint::from_u64(1_000_003);
    /// let e = p.sub(&Uint::one());
    /// assert_eq!(Uint::mod_pow(&Uint::from_u64(2), &e, &p), Uint::one());
    /// ```
    pub fn mod_pow(base: &Uint, exponent: &Uint, modulus: &Uint) -> Uint {
        if modulus.is_zero() {
            panic!(
                "Uint::mod_pow called with a zero modulus (base {}, exponent {})",
                base.to_dec_string(),
                exponent.to_dec_string()
            );
        }
        if modulus.is_one() {
            return Uint::zero();
        }
        let mut result = Uint::one();
        let mut square = base.rem(modulus);
        for index in 0..exponent.bit_len() {
            if exponent.bit(index) {
                result = result.mul_mod(&square, modulus);
            }
            // Skip the final, unused squaring.
            if index + 1 < exponent.bit_len() {
                square = square.mul_mod(&square, modulus);
            }
        }
        result
    }

    /// Computes `self^exponent mod modulus`, a method form of
    /// [`Uint::mod_pow`].
    ///
    /// # Panics
    ///
    /// Panics when `modulus` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// let got = Uint::from_u64(7).pow_mod(&Uint::from_u64(2), &Uint::from_u64(11));
    /// assert_eq!(got, Uint::from_u64(5));
    /// ```
    pub fn pow_mod(&self, exponent: &Uint, modulus: &Uint) -> Uint {
        Uint::mod_pow(self, exponent, modulus)
    }
}
