//! # Addition and checked subtraction
//!
//! The public [`BigUint::add`] and [`BigUint::sub`] wrappers over the limb loops
//! in the `carry` module, which documents the carry and borrow rules in detail.
//!
//! **Subtraction is checked, not saturating.** `2 - 5` is
//! [`BigUintError::Underflow`], not zero. Saturating would let an RSA padding
//! check quietly succeed on a malformed value, so the error is deliberate. Use
//! [`BigUint::sub_unchecked_wrapping`] only where the caller has already proven
//! `a >= b`, as the division inner loop has.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::{BigUint, BigUintError};
//!
//! // Carry through several limbs: (2^128 - 1) + 1 == 2^128.
//! let max = BigUint::from_limbs_le(vec![u64::MAX, u64::MAX]);
//! assert_eq!(max.add(&BigUint::from_u64(1)).limbs(), &[0, 0, 1]);
//!
//! // Borrow across a zero limb: 2^128 - 1.
//! let two_pow_128 = BigUint::from_limbs_le(vec![0, 0, 1]);
//! let less_one = two_pow_128.sub(&BigUint::from_u64(1)).unwrap();
//! assert_eq!(less_one.limbs(), &[u64::MAX, u64::MAX]);
//!
//! assert_eq!(
//!     BigUint::from_u64(2).sub(&BigUint::from_u64(5)),
//!     Err(BigUintError::Underflow)
//! );
//! ```

use std::cmp::Ordering;

use super::carry::{add_limbs, sub_limbs};
use super::error::BigUintError;
use super::limbs::BigUint;

impl BigUint {
    /// Add two values.
    ///
    /// # Arguments
    ///
    /// * `other` — the addend.
    ///
    /// # Returns
    ///
    /// A normalized `self + other`. Addition never fails: the result simply
    /// grows a limb when the top limb carries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// let a = BigUint::from_u64(u64::MAX);
    /// assert_eq!(a.add(&BigUint::from_u64(1)).limbs(), &[0, 1]);
    /// assert_eq!(a.add(&BigUint::zero()), a);
    /// ```
    pub fn add(&self, other: &Self) -> Self {
        Self::normalized(add_limbs(&self.limbs, &other.limbs))
    }

    /// Subtract `other` from `self`, checked against unsigned underflow.
    ///
    /// # Arguments
    ///
    /// * `other` — the subtrahend.
    ///
    /// # Returns
    ///
    /// A normalized `self - other`.
    ///
    /// # Errors
    ///
    /// [`BigUintError::Underflow`] when `other > self`, since the result cannot
    /// be represented unsigned. Nothing is truncated or saturated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::{BigUint, BigUintError};
    ///
    /// let a = BigUint::from_u64(1_000);
    /// assert_eq!(a.sub(&BigUint::from_u64(24)).unwrap(), BigUint::from_u64(976));
    /// assert_eq!(a.sub(&a).unwrap(), BigUint::zero());
    /// assert_eq!(BigUint::zero().sub(&a), Err(BigUintError::Underflow));
    /// ```
    pub fn sub(&self, other: &Self) -> Result<Self, BigUintError> {
        if self.compare(other) == Ordering::Less {
            return Err(BigUintError::Underflow);
        }
        Ok(self.sub_unchecked_wrapping(other))
    }

    /// Subtract without the ordering check, wrapping modulo `2^(64 * limbs)`.
    ///
    /// # Arguments
    ///
    /// * `other` — the subtrahend, which the caller must have proven is `<=
    ///   self`.
    ///
    /// # Returns
    ///
    /// A normalized `self - other` when the precondition holds; a meaningless
    /// wrapped value otherwise. Prefer [`BigUint::sub`] unless the comparison is
    /// already done, as in the long-division inner loop.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// let a = BigUint::from_u64(7);
    /// assert_eq!(a.sub_unchecked_wrapping(&BigUint::from_u64(4)), BigUint::from_u64(3));
    /// ```
    pub fn sub_unchecked_wrapping(&self, other: &Self) -> Self {
        Self::normalized(sub_limbs(&self.limbs, &other.limbs))
    }
}
