//! Subtraction for [`Uint`], with explicit underflow behaviour.
//!
//! # Underflow policy
//!
//! `Uint` is unsigned, so `a - b` with `a < b` has no representable answer.
//! This module refuses to silently wrap:
//!
//! * [`Uint::checked_sub`] returns `None` — use it whenever underflow is
//!   possible.
//! * [`Uint::sub`] panics with a message naming both operands — it is the
//!   convenience wrapper for places (division, Montgomery-free modular
//!   reduction) where the caller has already proved `self >= other`.
//!
//! No wrapping or saturating variant is provided, deliberately: a modular
//! reduction bug that quietly saturates to zero would be invisible, whereas a
//! panic is not.
//!
//! Borrows are propagated in `u128` so the `lhs + 2^64 - rhs` fixup is exact.

use super::uint::Uint;

impl Uint {
    /// Subtracts `other` from `self`, returning `None` on underflow.
    ///
    /// # Arguments
    ///
    /// * `other` — the subtrahend.
    ///
    /// # Returns
    ///
    /// `Some(self - other)` when `self >= other`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::from_u64(5).checked_sub(&Uint::from_u64(3)), Some(Uint::from_u64(2)));
    /// // Underflow is reported, not wrapped.
    /// assert_eq!(Uint::from_u64(3).checked_sub(&Uint::from_u64(5)), None);
    /// // Borrow across a limb boundary.
    /// let two_limbs = Uint::from_limbs(vec![0, 1]);
    /// assert_eq!(two_limbs.checked_sub(&Uint::one()), Some(Uint::from_u64(u64::MAX)));
    /// ```
    pub fn checked_sub(&self, other: &Uint) -> Option<Uint> {
        if self.cmp_uint(other).is_lt() {
            return None;
        }
        let mut out = Vec::with_capacity(self.limbs.len());
        let mut borrow: u64 = 0;
        for index in 0..self.limbs.len() {
            let lhs = self.limb(index) as u128;
            // At most (2^64 - 1) + 1 == 2^64, which is why this is u128.
            let rhs = other.limb(index) as u128 + borrow as u128;
            if lhs >= rhs {
                out.push((lhs - rhs) as u64);
                borrow = 0;
            } else {
                out.push((lhs + (1u128 << 64) - rhs) as u64);
                borrow = 1;
            }
        }
        debug_assert_eq!(borrow, 0, "borrow must clear when self >= other");
        Some(Uint::from_limbs(out))
    }

    /// Subtracts `other` from `self`.
    ///
    /// # Panics
    ///
    /// Panics when `other > self`, naming both operands in decimal. Use
    /// [`Uint::checked_sub`] when underflow is a possible outcome.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::from_u64(9).sub(&Uint::from_u64(4)), Uint::from_u64(5));
    /// ```
    // Inherent, not `std::ops::Sub`: the trait signature cannot express the
    // borrowed operands, and a panicking operator would be a trap.
    #[allow(clippy::should_implement_trait)]
    pub fn sub(&self, other: &Uint) -> Uint {
        match self.checked_sub(other) {
            Some(value) => value,
            None => panic!(
                "Uint underflow: {} - {} is negative and unsigned integers cannot represent it",
                self.to_dec_string(),
                other.to_dec_string()
            ),
        }
    }
}
