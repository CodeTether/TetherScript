//! Bit inspection for [`Uint`]: bit length and individual bit tests.
//!
//! Bit length relies on the normalization invariant. If trailing zero limbs
//! were allowed, `bit_len` would count leading zeros of a zero top limb and
//! report a wildly wrong 64-bit-aligned value, which in turn would make
//! [`Uint::mod_pow`]'s square-and-multiply loop scan garbage bits.

use super::uint::Uint;

impl Uint {
    /// Returns the number of bits needed to represent the value.
    ///
    /// # Returns
    ///
    /// `0` for zero, otherwise `floor(log2(self)) + 1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::zero().bit_len(), 0);
    /// assert_eq!(Uint::one().bit_len(), 1);
    /// assert_eq!(Uint::from_u64(255).bit_len(), 8);
    /// assert_eq!(Uint::from_limbs(vec![0, 1]).bit_len(), 65);
    /// ```
    pub fn bit_len(&self) -> usize {
        match self.limbs.last() {
            None => 0,
            Some(&top) => self.limbs.len() * 64 - top.leading_zeros() as usize,
        }
    }

    /// Tests bit `index`, counting from the least significant bit.
    ///
    /// # Arguments
    ///
    /// * `index` — zero-based bit position.
    ///
    /// # Returns
    ///
    /// `true` when the bit is set. Indices at or beyond [`Uint::bit_len`]
    /// return `false` rather than panicking, so callers can scan a fixed width.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// let five = Uint::from_u64(0b101);
    /// assert!(five.bit(0));
    /// assert!(!five.bit(1));
    /// assert!(five.bit(2));
    /// assert!(!five.bit(4096));
    /// ```
    pub fn bit(&self, index: usize) -> bool {
        let limb = index / 64;
        if limb >= self.limbs.len() {
            return false;
        }
        (self.limbs[limb] >> (index % 64)) & 1 == 1
    }
}
