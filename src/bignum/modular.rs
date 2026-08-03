//! Modular arithmetic helpers used by [`Uint::mod_pow`].
//!
//! Reduction is done with full long division (`Uint::rem`). That is the
//! straightforward, auditable choice: a Montgomery ladder would be faster but
//! introduces a residue representation, an odd-modulus precondition, and an
//! `n'` inverse to get wrong. Correctness first — the module's own docs say so.

use super::uint::Uint;

impl Uint {
    /// Reduces `self` modulo `modulus`.
    ///
    /// # Arguments
    ///
    /// * `modulus` — a non-zero modulus.
    ///
    /// # Returns
    ///
    /// `self mod modulus`, always strictly less than `modulus`.
    ///
    /// # Panics
    ///
    /// Panics when `modulus` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::from_u64(10).mod_reduce(&Uint::from_u64(7)), Uint::from_u64(3));
    /// assert!(Uint::from_u64(10).mod_reduce(&Uint::one()).is_zero());
    /// ```
    pub fn mod_reduce(&self, modulus: &Uint) -> Uint {
        self.rem(modulus)
    }

    /// Computes `(self * other) mod modulus`.
    ///
    /// The full `2n`-limb product is formed first and then reduced, so no
    /// precision is lost regardless of how large the operands are.
    ///
    /// # Arguments
    ///
    /// * `other` — the multiplicand.
    /// * `modulus` — a non-zero modulus.
    ///
    /// # Returns
    ///
    /// The product reduced modulo `modulus`.
    ///
    /// # Panics
    ///
    /// Panics when `modulus` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// let m = Uint::from_u64(97);
    /// assert_eq!(
    ///     Uint::from_u64(50).mul_mod(&Uint::from_u64(50), &m),
    ///     Uint::from_u64(2500 % 97)
    /// );
    /// ```
    pub fn mul_mod(&self, other: &Uint, modulus: &Uint) -> Uint {
        self.mul(other).rem(modulus)
    }

    /// Computes `(self + other) mod modulus`.
    ///
    /// # Panics
    ///
    /// Panics when `modulus` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// let m = Uint::from_u64(7);
    /// assert_eq!(Uint::from_u64(5).add_mod(&Uint::from_u64(4), &m), Uint::from_u64(2));
    /// ```
    pub fn add_mod(&self, other: &Uint, modulus: &Uint) -> Uint {
        self.add(other).rem(modulus)
    }
}
