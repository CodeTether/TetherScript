//! # Modular reduction and modular multiplication
//!
//! The two helpers [`BigUint::rem`] and [`BigUint::mulmod`] that
//! [`BigUint::modpow`] is built from. Both are thin, deliberate wrappers over
//! [`BigUint::divmod`]: multiply into the full `2n`-limb product first, then
//! reduce. Reducing after every multiply is what keeps intermediate values
//! bounded by the modulus instead of doubling in width on every squaring.
//!
//! No Montgomery form. It would be faster, but it needs an odd modulus and an
//! extra conversion in and out, and this module is the reference these
//! optimizations would have to be checked against.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUint;
//!
//! let modulus = BigUint::from_u64(497);
//! assert_eq!(BigUint::from_u64(1_000).rem(&modulus).unwrap(), BigUint::from_u64(6));
//! let four = BigUint::from_u64(4);
//! // 4 * 4 mod 497
//! assert_eq!(four.mulmod(&four, &modulus).unwrap(), BigUint::from_u64(16));
//! ```

use super::error::BigUintError;
use super::limbs::BigUint;

impl BigUint {
    /// Reduce modulo `modulus`.
    ///
    /// # Arguments
    ///
    /// * `modulus` — must be nonzero.
    ///
    /// # Returns
    ///
    /// `self mod modulus`, normalized and strictly less than `modulus`.
    ///
    /// # Errors
    ///
    /// [`BigUintError::DivideByZero`] when `modulus` is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert_eq!(
    ///     BigUint::from_u64(10).rem(&BigUint::from_u64(3)).unwrap(),
    ///     BigUint::from_u64(1)
    /// );
    /// assert!(BigUint::from_u64(9).rem(&BigUint::from_u64(3)).unwrap().is_zero());
    /// ```
    pub fn rem(&self, modulus: &Self) -> Result<Self, BigUintError> {
        self.divmod(modulus).map(|(_, remainder)| remainder)
    }

    /// Multiply and reduce in one step: `self * other mod modulus`.
    ///
    /// # Arguments
    ///
    /// * `other` — the multiplicand.
    /// * `modulus` — must be nonzero.
    ///
    /// # Returns
    ///
    /// The normalized product modulo `modulus`. Operands need not already be
    /// reduced; the full product is formed before the reduction, so nothing
    /// wraps.
    ///
    /// # Errors
    ///
    /// [`BigUintError::DivideByZero`] when `modulus` is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// let a = BigUint::from_u64(u64::MAX);
    /// let modulus = BigUint::from_u64(1_000);
    /// // (2^64 - 1)^2 mod 1000 == 225
    /// assert_eq!(a.mulmod(&a, &modulus).unwrap(), BigUint::from_u64(225));
    /// ```
    pub fn mulmod(&self, other: &Self, modulus: &Self) -> Result<Self, BigUintError> {
        self.mul(other).rem(modulus)
    }
}
