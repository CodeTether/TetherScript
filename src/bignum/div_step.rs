//! Division of a [`Uint`] by a single 64-bit limb.
//!
//! A top-down sweep: carry the running remainder into the high half of a `u128`
//! and divide by the limb. The dividend of each step is
//! `rem * 2^64 + limb[i]`, which needs 128 bits; since `rem < divisor <= 2^64-1`
//! the quotient digit always fits in a `u64`.

use super::uint::Uint;

impl Uint {
    /// Divides by a single limb, returning the quotient and remainder.
    ///
    /// # Arguments
    ///
    /// * `divisor` — a non-zero 64-bit divisor.
    ///
    /// # Returns
    ///
    /// `(quotient, remainder)` where `remainder < divisor`.
    ///
    /// # Panics
    ///
    /// Panics when `divisor` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// let (q, r) = Uint::from_u64(1_000).divmod_u64(7);
    /// assert_eq!(q, Uint::from_u64(142));
    /// assert_eq!(r, 6);
    ///
    /// // 2^64 / 10
    /// let (q, r) = Uint::from_limbs(vec![0, 1]).divmod_u64(10);
    /// assert_eq!(q, Uint::from_u64(1_844_674_407_370_955_161));
    /// assert_eq!(r, 6);
    /// ```
    pub fn divmod_u64(&self, divisor: u64) -> (Uint, u64) {
        if divisor == 0 {
            panic!("Uint division by zero: {} / 0", self.to_dec_string());
        }
        let mut quotient = vec![0u64; self.limbs.len()];
        let mut rem: u64 = 0;
        // Most significant limb first, so `rem` is the carry from above.
        for (slot, &limb) in quotient.iter_mut().zip(&self.limbs).rev() {
            let cur = ((rem as u128) << 64) | limb as u128;
            *slot = (cur / divisor as u128) as u64;
            rem = (cur % divisor as u128) as u64;
        }
        (Uint::from_limbs(quotient), rem)
    }
}
