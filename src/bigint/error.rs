//! # Named arithmetic errors
//!
//! Every fallible operation on [`BigUint`](crate::bigint::BigUint) reports a variant of
//! [`BigUintError`] instead of panicking. Division by zero and a too-narrow
//! fixed-width encoding are *caller* mistakes that a library must be able to
//! surface across an FFI or capability boundary, so they are values, not
//! aborts.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::{BigUint, BigUintError};
//!
//! let err = BigUint::from_u64(1).divmod(&BigUint::zero()).unwrap_err();
//! assert_eq!(err, BigUintError::DivideByZero);
//! assert!(err.to_string().contains("divide by zero"));
//! ```

use std::fmt;

/// Why a [`BigUint`](crate::bigint::BigUint) operation could not produce a value.
///
/// Each variant carries the numbers needed to explain the failure, because an
/// error that does not name the thing that went wrong is not an error message.
///
/// # Examples
///
/// ```rust
/// use tetherscript::bigint::{BigUint, BigUintError};
///
/// // A fixed-width encoding that cannot hold the value is refused, never
/// // silently truncated.
/// let err = BigUint::from_u64(0x1_0000).to_be_bytes(2).unwrap_err();
/// assert_eq!(
///     err,
///     BigUintError::WidthTooSmall { needed: 3, width: 2 }
/// );
///
/// // Subtraction is checked: unsigned values cannot go negative.
/// let err = BigUint::from_u64(1).sub(&BigUint::from_u64(2)).unwrap_err();
/// assert_eq!(err, BigUintError::Underflow);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BigUintError {
    /// [`BigUint::divmod`](crate::bigint::BigUint::divmod) or
    /// [`BigUint::modpow`](crate::bigint::BigUint::modpow) was given a zero modulus.
    DivideByZero,
    /// [`BigUint::to_be_bytes`](crate::bigint::BigUint::to_be_bytes) was asked for a
    /// `width` smaller than the `needed` minimum byte length of the value.
    WidthTooSmall {
        /// Minimum byte count that represents the value.
        needed: usize,
        /// Requested output width, in bytes.
        width: usize,
    },
    /// [`BigUint::sub`](crate::bigint::BigUint::sub) would have produced a negative
    /// result, which an unsigned type cannot represent.
    Underflow,
}

impl fmt::Display for BigUintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DivideByZero => write!(f, "bigint: divide by zero"),
            Self::WidthTooSmall { needed, width } => write!(
                f,
                "bigint: value needs {needed} bytes but output width is {width}"
            ),
            Self::Underflow => write!(
                f,
                "bigint: subtraction underflow (unsigned result would be negative)"
            ),
        }
    }
}

impl std::error::Error for BigUintError {}
