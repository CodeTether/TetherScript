//! Addition for [`Uint`].
//!
//! Schoolbook limb addition with a carry. Each limb sum can be up to
//! `2*(2^64-1)+1`, which overflows `u64`, so the sum is formed in `u128` and
//! split into a low limb plus a 0/1 carry.

use super::uint::Uint;

impl Uint {
    /// Adds two values.
    ///
    /// Never overflows: the result grows a limb when it has to.
    ///
    /// # Arguments
    ///
    /// * `other` — the addend.
    ///
    /// # Returns
    ///
    /// `self + other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// // Carry out of the low limb creates a second limb.
    /// let max = Uint::from_u64(u64::MAX);
    /// assert_eq!(max.add(&Uint::one()).limbs(), &[0, 1]);
    /// assert_eq!(Uint::from_u64(2).add(&Uint::from_u64(3)), Uint::from_u64(5));
    /// assert_eq!(Uint::zero().add(&max), max);
    /// ```
    // Inherent, not `std::ops::Add`: the operands are borrowed rather than
    // consumed, since a 2048-bit value is expensive to clone into an operator.
    #[allow(clippy::should_implement_trait)]
    pub fn add(&self, other: &Uint) -> Uint {
        let len = self.limbs.len().max(other.limbs.len());
        let mut out = Vec::with_capacity(len + 1);
        let mut carry: u64 = 0;
        for index in 0..len {
            // u128 keeps the intermediate exact; u64 arithmetic would wrap.
            let sum = self.limb(index) as u128 + other.limb(index) as u128 + carry as u128;
            out.push(sum as u64);
            carry = (sum >> 64) as u64;
        }
        if carry != 0 {
            out.push(carry);
        }
        Uint::from_limbs(out)
    }

    /// Adds a single limb.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::from_u64(10).add_u64(5), Uint::from_u64(15));
    /// ```
    pub fn add_u64(&self, value: u64) -> Uint {
        self.add(&Uint::from_u64(value))
    }
}
