//! Division entry points for [`Uint`]: [`Uint::divmod`], [`Uint::div`], and
//! [`Uint::rem`].
//!
//! # Algorithm choice
//!
//! The multi-limb path is **Knuth Algorithm D** (TAOCP vol. 2, §4.3.1), the
//! reference schoolbook long-division algorithm, implemented in the private
//! `div_knuth` sibling module. It was chosen over the simpler bit-by-bit
//! shift-and-subtract loop because a 2048-bit `mod_pow` performs on the order of
//! 3000 reductions of a 4096-bit intermediate; bit-by-bit division costs one
//! multi-limb compare-and-subtract per *bit* (~4096 of them per division),
//! whereas Algorithm D costs one per *limb* of the quotient (~32). That is
//! roughly a 64x difference on the hot path, which is the difference between a
//! usable JWT verifier and an unusable one.
//!
//! Two shortcuts precede it, because Algorithm D requires a divisor of at least
//! two limbs:
//!
//! * `self < other` — the quotient is zero and the remainder is `self`.
//! * single-limb divisor — a straight `u128 / u64` sweep from the top limb down.

use super::uint::Uint;

impl Uint {
    /// Divides with remainder.
    ///
    /// # Arguments
    ///
    /// * `other` — the divisor.
    ///
    /// # Returns
    ///
    /// `(quotient, remainder)` with `self == quotient * other + remainder` and
    /// `remainder < other`.
    ///
    /// # Panics
    ///
    /// Panics when `other` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// let (q, r) = Uint::from_u64(17).divmod(&Uint::from_u64(5));
    /// assert_eq!(q, Uint::from_u64(3));
    /// assert_eq!(r, Uint::from_u64(2));
    ///
    /// // Divisor larger than the dividend.
    /// let (q, r) = Uint::from_u64(5).divmod(&Uint::from_u64(17));
    /// assert!(q.is_zero());
    /// assert_eq!(r, Uint::from_u64(5));
    /// ```
    pub fn divmod(&self, other: &Uint) -> (Uint, Uint) {
        if other.is_zero() {
            panic!("Uint division by zero: {} / 0", self.to_dec_string());
        }
        if self.cmp_uint(other).is_lt() {
            return (Uint::zero(), self.clone());
        }
        if other.limbs.len() == 1 {
            let (q, r) = self.divmod_u64(other.limbs[0]);
            return (q, Uint::from_u64(r));
        }
        super::div_knuth::divmod_knuth(self, other)
    }

    /// Returns the quotient `floor(self / other)`.
    ///
    /// # Panics
    ///
    /// Panics when `other` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::from_u64(100).div(&Uint::from_u64(7)), Uint::from_u64(14));
    /// ```
    // Inherent, not `std::ops::Div`, so operands stay borrowed.
    #[allow(clippy::should_implement_trait)]
    pub fn div(&self, other: &Uint) -> Uint {
        self.divmod(other).0
    }

    /// Returns the remainder `self mod other`.
    ///
    /// # Panics
    ///
    /// Panics when `other` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::from_u64(100).rem(&Uint::from_u64(7)), Uint::from_u64(2));
    /// assert!(Uint::from_u64(100).rem(&Uint::one()).is_zero());
    /// ```
    // Inherent, not `std::ops::Rem`, so operands stay borrowed.
    #[allow(clippy::should_implement_trait)]
    pub fn rem(&self, other: &Uint) -> Uint {
        self.divmod(other).1
    }
}
