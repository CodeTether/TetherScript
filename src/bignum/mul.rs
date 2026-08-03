//! Multiplication for [`Uint`].
//!
//! # Why `u128` is required, and why it is enough
//!
//! The product of two limbs is up to `(2^64 - 1)^2 = 2^128 - 2^65 + 1`, which
//! does not fit in a `u64`. The inner loop therefore accumulates in `u128`:
//!
//! ```text
//! t = a[i] * b[j] + out[i + j] + carry
//! ```
//!
//! The worst case is exactly representable:
//! `(2^64 - 1)^2 + (2^64 - 1) + (2^64 - 1) = 2^128 - 1`. So the accumulator can
//! never overflow `u128`, and `carry = t >> 64` always fits back in a `u64`.
//! This is schoolbook `O(n*m)` multiplication; no Karatsuba, since 2048-bit RSA
//! (32 limbs) is well below the crossover where it pays off.

use super::uint::Uint;

impl Uint {
    /// Multiplies two values.
    ///
    /// # Arguments
    ///
    /// * `other` — the multiplicand.
    ///
    /// # Returns
    ///
    /// `self * other`. Zero times anything is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::from_u64(6).mul(&Uint::from_u64(7)), Uint::from_u64(42));
    /// assert_eq!(Uint::from_u64(12345).mul(&Uint::zero()), Uint::zero());
    ///
    /// // Crossing the limb boundary: 2^32 * 2^32 == 2^64 == limbs [0, 1].
    /// let root = Uint::from_u64(1 << 32);
    /// assert_eq!(root.mul(&root).limbs(), &[0, 1]);
    /// ```
    // Inherent, not `std::ops::Mul`, so operands stay borrowed.
    #[allow(clippy::should_implement_trait)]
    pub fn mul(&self, other: &Uint) -> Uint {
        if self.is_zero() || other.is_zero() {
            return Uint::zero();
        }
        let mut out = vec![0u64; self.limbs.len() + other.limbs.len()];
        for (i, &a) in self.limbs.iter().enumerate() {
            let mut carry: u64 = 0;
            for (j, &b) in other.limbs.iter().enumerate() {
                let t = a as u128 * b as u128 + out[i + j] as u128 + carry as u128;
                out[i + j] = t as u64;
                carry = (t >> 64) as u64;
            }
            // The reserved high half guarantees this index exists.
            out[i + other.limbs.len()] = carry;
        }
        Uint::from_limbs(out)
    }

    /// Multiplies by a single limb.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::from_u64(21).mul_u64(2), Uint::from_u64(42));
    /// ```
    pub fn mul_u64(&self, value: u64) -> Uint {
        self.mul(&Uint::from_u64(value))
    }
}
