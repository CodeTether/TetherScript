//! Left and right bit shifts for [`Uint`].
//!
//! Both are implemented as a whole-limb offset plus an intra-limb shift. The
//! intra-limb amount is guarded to be non-zero before shifting by
//! `64 - bit_shift`, because shifting a `u64` by 64 is undefined in Rust (it
//! panics in debug builds and is a no-op on some targets in release).
//!
//! Left shift never loses bits — the result simply grows. Right shift discards
//! the bits shifted out, matching `>>` on primitive unsigned integers, so it is
//! exactly floor-division by a power of two.

use super::uint::Uint;

impl Uint {
    /// Shifts left by `bits`, multiplying by `2^bits`.
    ///
    /// # Arguments
    ///
    /// * `bits` — how many bit positions to shift.
    ///
    /// # Returns
    ///
    /// `self * 2^bits`. Zero shifts to zero for any `bits`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::one().shl(64).limbs(), &[0, 1]);
    /// assert_eq!(Uint::from_u64(3).shl(2), Uint::from_u64(12));
    /// assert_eq!(Uint::zero().shl(999), Uint::zero());
    /// assert_eq!(Uint::one().shl(65).bit_len(), 66);
    /// ```
    pub fn shl(&self, bits: usize) -> Uint {
        if self.is_zero() {
            return Uint::zero();
        }
        let (limb_shift, bit_shift) = (bits / 64, bits % 64);
        let mut out = vec![0u64; self.limbs.len() + limb_shift + 1];
        for (i, &limb) in self.limbs.iter().enumerate() {
            out[i + limb_shift] |= limb << bit_shift;
            if bit_shift > 0 {
                out[i + limb_shift + 1] |= limb >> (64 - bit_shift);
            }
        }
        Uint::from_limbs(out)
    }

    /// Shifts right by `bits`, flooring the division by `2^bits`.
    ///
    /// # Arguments
    ///
    /// * `bits` — how many bit positions to shift; bits shifted out are lost.
    ///
    /// # Returns
    ///
    /// `floor(self / 2^bits)`, which is zero once `bits >= self.bit_len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::from_u64(12).shr(2), Uint::from_u64(3));
    /// assert_eq!(Uint::from_u64(13).shr(2), Uint::from_u64(3));
    /// assert_eq!(Uint::from_limbs(vec![0, 1]).shr(64), Uint::one());
    /// assert_eq!(Uint::from_u64(5).shr(99), Uint::zero());
    /// ```
    pub fn shr(&self, bits: usize) -> Uint {
        let (limb_shift, bit_shift) = (bits / 64, bits % 64);
        if limb_shift >= self.limbs.len() {
            return Uint::zero();
        }
        let len = self.limbs.len() - limb_shift;
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            let mut value = self.limb(i + limb_shift) >> bit_shift;
            if bit_shift > 0 {
                value |= self.limb(i + limb_shift + 1) << (64 - bit_shift);
            }
            out.push(value);
        }
        Uint::from_limbs(out)
    }
}
