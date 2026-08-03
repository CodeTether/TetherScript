//! # Shifting by one bit
//!
//! The single shift primitive the rest of the crate needs:
//! [`BigUint::double_plus_bit`], which computes `self * 2 + bit`. The
//! long-division loop in [`BigUint::divmod`] uses it to feed the dividend's bits
//! into the running remainder one at a time.
//!
//! Only the low bit of each limb crosses a boundary, so each step is
//! `(limb << 1) | carry_in`, with `carry_out = limb >> 63`. As in
//! the `carry` module, the carry must be propagated all the way up and may grow
//! the vector by one limb.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUint;
//!
//! // Doubling the largest single limb spills into a second limb.
//! let value = BigUint::from_u64(u64::MAX);
//! assert_eq!(value.double_plus_bit(false).limbs(), &[u64::MAX - 1, 1]);
//! assert_eq!(value.double_plus_bit(true).limbs(), &[u64::MAX, 1]);
//!
//! // Zero stays canonical when no bit is shifted in.
//! assert!(BigUint::zero().double_plus_bit(false).is_zero());
//! assert!(BigUint::zero().double_plus_bit(true).is_one());
//! ```

use super::limbs::BigUint;

impl BigUint {
    /// Compute `self * 2 + bit`, a left shift by one with a bit shifted in.
    ///
    /// # Arguments
    ///
    /// * `bit` — the value of the new least significant bit.
    ///
    /// # Returns
    ///
    /// A normalized `self * 2 + bit`. Never fails; the vector grows a limb when
    /// the top bit of the top limb is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// let three = BigUint::from_u64(3);
    /// assert_eq!(three.double_plus_bit(true), BigUint::from_u64(7));
    /// assert_eq!(three.double_plus_bit(false), BigUint::from_u64(6));
    /// ```
    pub fn double_plus_bit(&self, bit: bool) -> Self {
        let mut out = Vec::with_capacity(self.limbs.len() + 1);
        let mut carry = u64::from(bit);
        for &limb in &self.limbs {
            out.push((limb << 1) | carry);
            carry = limb >> 63;
        }
        if carry != 0 {
            out.push(carry);
        }
        Self::normalized(out)
    }
}
