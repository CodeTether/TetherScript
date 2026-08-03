//! The [`ParseUintError`] type returned by [`Uint`](super::uint::Uint) string
//! parsing.
//!
//! Errors name the offending character and its position, per the project's rule
//! that every error path identifies what went wrong.

use std::fmt;

/// Why a decimal or hex string could not be parsed as a `Uint`.
///
/// # Examples
///
/// ```
/// use tetherscript::bignum::{ParseUintError, Uint};
///
/// match Uint::from_dec_str("12x4") {
///     Err(ParseUintError::InvalidDigit { ch, index }) => {
///         assert_eq!(ch, 'x');
///         assert_eq!(index, 2);
///     }
///     other => panic!("expected an invalid-digit error, got {other:?}"),
/// }
/// assert!(matches!(Uint::from_hex_str(""), Err(ParseUintError::Empty)));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseUintError {
    /// The input had no digits at all (after an optional `0x` prefix).
    Empty,
    /// A character was not a digit in the requested radix.
    InvalidDigit {
        /// The offending character.
        ch: char,
        /// Its zero-based index in the original input.
        index: usize,
    },
}

impl fmt::Display for ParseUintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "cannot parse an empty string as a Uint"),
            Self::InvalidDigit { ch, index } => {
                write!(f, "invalid digit {ch:?} at index {index} while parsing a Uint")
            }
        }
    }
}

impl std::error::Error for ParseUintError {}
