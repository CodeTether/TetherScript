//! # Bit-level queries
//!
//! Bit length and single-bit access. [`BigUint::modpow`] walks exponent bits
//! from the most significant downward, so both are on the hot path of RSA.
//!
//! Bit `i` is bit `i % 64` of limb `i / 64`, following the little-endian limb
//! order documented in the `limbs` module.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUint;
//!
//! let value = BigUint::from_u64(0b1011);
//! assert_eq!(value.bit_len(), 4);
//! assert!(value.bit(0));
//! assert!(!value.bit(2));
//! assert!(!value.bit(9_999), "out-of-range bits read as zero");
//! ```

use super::limbs::BigUint;

impl BigUint {
    /// Number of significant bits, i.e. the position of the highest set bit
    /// plus one.
    ///
    /// # Returns
    ///
    /// `0` for zero, otherwise `64 * (limbs - 1) + (64 - top.leading_zeros())`.
    /// Correct only because the value is normalized: a trailing zero limb would
    /// inflate the answer by 64. See the `limbs` module.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert_eq!(BigUint::zero().bit_len(), 0);
    /// assert_eq!(BigUint::from_u64(1).bit_len(), 1);
    /// assert_eq!(BigUint::from_u64(u64::MAX).bit_len(), 64);
    /// assert_eq!(BigUint::from_limbs_le(vec![0, 1]).bit_len(), 65);
    /// ```
    pub fn bit_len(&self) -> usize {
        match self.limbs.last() {
            None => 0,
            Some(top) => self.limbs.len() * 64 - top.leading_zeros() as usize,
        }
    }

    /// Read a single bit.
    ///
    /// # Arguments
    ///
    /// * `index` — zero-based bit position, `0` being the least significant.
    ///
    /// # Returns
    ///
    /// `true` when the bit is set. Positions at or beyond [`BigUint::bit_len`]
    /// read as `false` rather than panicking, so exponent scans need no bounds
    /// juggling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// let value = BigUint::from_limbs_le(vec![0, 1]); // 2^64
    /// assert!(value.bit(64));
    /// assert!(!value.bit(63));
    /// ```
    pub fn bit(&self, index: usize) -> bool {
        let limb = index / 64;
        match self.limbs.get(limb) {
            None => false,
            Some(word) => ((word >> (index % 64)) & 1) == 1,
        }
    }
}
