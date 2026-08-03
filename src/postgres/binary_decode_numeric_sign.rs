//! # `numeric` sign words
//!
//! The sign field is not a sign *bit* — it is one of five specific 16-bit words, and
//! two of them (`NaN`, and the infinities added in PostgreSQL 14) mean the value has
//! **no digit groups at all**. So the sign must be inspected before any attempt to read
//! digits, or a NaN looks like a truncated frame.
//!
//! An unrecognised word is rejected rather than treated as positive. Guessing here
//! would turn a corrupt frame into a plausible positive number.

use super::super::super::error::DecodeError;
use super::header::Header;

/// Sign word for a positive value (`NUMERIC_POS`).
const SIGN_POS: u16 = 0x0000;
/// Sign word for a negative value (`NUMERIC_NEG`).
const SIGN_NEG: u16 = 0x4000;
/// Sign word for NaN (`NUMERIC_NAN`); carries no digit groups.
const SIGN_NAN: u16 = 0xC000;
/// Sign word for `Infinity` (`NUMERIC_PINF`, PostgreSQL 14+).
const SIGN_PINF: u16 = 0xD000;
/// Sign word for `-Infinity` (`NUMERIC_NINF`, PostgreSQL 14+).
const SIGN_NINF: u16 = 0xF000;

/// Reject a sign word that is none of the five documented values.
///
/// # Arguments
///
/// * `sign` — the raw big-endian sign word.
///
/// # Errors
///
/// [`DecodeError::BadNumericSign`], naming the offending word in hex.
pub(super) fn validate(sign: u16) -> Result<(), DecodeError> {
    if matches!(sign, SIGN_POS | SIGN_NEG | SIGN_NAN | SIGN_PINF | SIGN_NINF) {
        return Ok(());
    }
    Err(DecodeError::BadNumericSign { sign })
}

impl Header {
    /// The literal to emit for a non-finite value.
    ///
    /// # Returns
    ///
    /// `Some("NaN" | "Infinity" | "-Infinity")` when the sign word marks a special
    /// value, and `None` for an ordinary finite number whose digits must be read. The
    /// spellings are the ones PostgreSQL emits and accepts back as input.
    pub(super) fn special(&self) -> Option<&'static str> {
        match self.sign {
            SIGN_NAN => Some("NaN"),
            SIGN_PINF => Some("Infinity"),
            SIGN_NINF => Some("-Infinity"),
            _ => None,
        }
    }

    /// Whether the value is negative. NaN and the infinities are never negative here;
    /// `-Infinity` carries its sign in its own literal instead.
    pub(super) fn negative(&self) -> bool {
        self.sign == SIGN_NEG
    }
}
