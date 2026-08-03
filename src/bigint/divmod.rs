//! # Long division: quotient and remainder together
//!
//! Full binary long division, the schoolbook algorithm one bit at a time. It
//! yields both the quotient and the remainder in one pass, since RSA needs the
//! remainder (modular reduction) far more often than the quotient and computing
//! them separately would double the work.
//!
//! ## The algorithm
//!
//! Walk the dividend's bits from the most significant down. At each step shift
//! that bit into a running remainder (via [`BigUint::double_plus_bit`]); if the
//! remainder has reached the divisor, subtract the divisor and set the
//! corresponding quotient bit. The remainder is therefore always strictly less
//! than the divisor at the top of each step, which is exactly what makes the
//! subtraction safe and lets it use [`BigUint::sub_unchecked_wrapping`] — the
//! comparison immediately above it has already proven `remainder >= divisor`.
//!
//! Cost is `O(bit_len(dividend) · limbs(divisor))`. No Knuth algorithm D limb
//! estimation: the bit loop has no quotient-digit correction step to get subtly
//! wrong, and correctness of the modular reduction under RSA matters more than
//! a constant factor.
//!
//! ## Division by zero is an error, not a panic
//!
//! A zero divisor returns [`BigUintError::DivideByZero`]. Panicking would take
//! down a host embedding the interpreter over what is an ordinary bad input.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::{BigUint, BigUintError};
//!
//! let (quotient, remainder) =
//!     BigUint::from_u64(1_000).divmod(&BigUint::from_u64(7)).unwrap();
//! assert_eq!(quotient, BigUint::from_u64(142));
//! assert_eq!(remainder, BigUint::from_u64(6));
//!
//! // Divisor larger than the dividend: quotient 0, remainder the dividend.
//! let (quotient, remainder) =
//!     BigUint::from_u64(7).divmod(&BigUint::from_u64(1_000)).unwrap();
//! assert!(quotient.is_zero());
//! assert_eq!(remainder, BigUint::from_u64(7));
//!
//! assert_eq!(
//!     BigUint::from_u64(1).divmod(&BigUint::zero()),
//!     Err(BigUintError::DivideByZero)
//! );
//! ```

use std::cmp::Ordering;

use super::error::BigUintError;
use super::limbs::BigUint;

impl BigUint {
    /// Divide by `divisor`, returning the quotient and remainder.
    ///
    /// # Arguments
    ///
    /// * `divisor` — the value to divide by; must be nonzero.
    ///
    /// # Returns
    ///
    /// `(quotient, remainder)` with `self == quotient * divisor + remainder` and
    /// `remainder < divisor`. Both are normalized, so an exact division yields a
    /// canonical zero remainder.
    ///
    /// # Errors
    ///
    /// [`BigUintError::DivideByZero`] when `divisor` is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// // (2^192 + 5) / (2^64 + 1)
    /// let dividend = BigUint::from_limbs_le(vec![5, 0, 0, 1]);
    /// let divisor = BigUint::from_limbs_le(vec![1, 1]);
    /// let (quotient, remainder) = dividend.divmod(&divisor).unwrap();
    /// assert_eq!(quotient.limbs(), &[1, u64::MAX]);
    /// assert_eq!(remainder, BigUint::from_u64(4));
    ///
    /// // Divisor equal to the dividend.
    /// let (quotient, remainder) = divisor.divmod(&divisor).unwrap();
    /// assert!(quotient.is_one() && remainder.is_zero());
    /// ```
    pub fn divmod(&self, divisor: &Self) -> Result<(Self, Self), BigUintError> {
        if divisor.is_zero() {
            return Err(BigUintError::DivideByZero);
        }
        if self.compare(divisor) == Ordering::Less {
            return Ok((Self::zero(), self.clone()));
        }
        let mut quotient = vec![0u64; self.limbs.len()];
        let mut remainder = Self::zero();
        for index in (0..self.bit_len()).rev() {
            remainder = remainder.double_plus_bit(self.bit(index));
            if remainder.compare(divisor) != Ordering::Less {
                remainder = remainder.sub_unchecked_wrapping(divisor);
                quotient[index / 64] |= 1u64 << (index % 64);
            }
        }
        Ok((Self::from_limbs_le(quotient), remainder))
    }
}
