//! Decimal and hexadecimal string parsing for [`Uint`].
//!
//! Both parsers ignore ASCII whitespace and `_` separators so that constants
//! copied out of specifications (RFC 3526 prints its primes as spaced hex
//! blocks) can be pasted verbatim. Everything else must be a digit in the
//! requested radix.
//!
//! Digits are folded in most-significant-first, one at a time:
//! `acc = acc * radix + digit`. That is `O(digits * limbs)`, which for a
//! 617-digit (2048-bit) decimal literal is a few thousand limb operations —
//! irrelevant next to the exponentiation it feeds.

use super::error::ParseUintError;
use super::uint::Uint;

impl Uint {
    /// Parses an unsigned decimal string.
    ///
    /// # Arguments
    ///
    /// * `text` — decimal digits; whitespace and `_` are ignored. No sign is
    ///   accepted, since `Uint` is unsigned.
    ///
    /// # Returns
    ///
    /// The parsed value. Leading zeros are fine: `"007"` is `7`.
    ///
    /// # Errors
    ///
    /// [`ParseUintError::Empty`] when no digits are present, or
    /// [`ParseUintError::InvalidDigit`] naming the first bad character.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::from_dec_str("007").unwrap(), Uint::from_u64(7));
    /// assert_eq!(Uint::from_dec_str("18446744073709551616").unwrap().limbs(), &[0, 1]);
    /// assert!(Uint::from_dec_str("-1").is_err());
    /// ```
    pub fn from_dec_str(text: &str) -> Result<Uint, ParseUintError> {
        fold(text, 10, |ch| ch.to_digit(10))
    }

    /// Parses an unsigned hexadecimal string, with an optional `0x` prefix.
    ///
    /// # Arguments
    ///
    /// * `text` — hex digits in either case; whitespace and `_` are ignored.
    ///
    /// # Errors
    ///
    /// [`ParseUintError::Empty`] when no digits follow the prefix, or
    /// [`ParseUintError::InvalidDigit`] naming the first bad character.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::from_hex_str("ff").unwrap(), Uint::from_u64(255));
    /// assert_eq!(Uint::from_hex_str("0xFF").unwrap(), Uint::from_u64(255));
    /// assert_eq!(Uint::from_hex_str("FFFF FFFF").unwrap(), Uint::from_u64(0xFFFF_FFFF));
    /// ```
    pub fn from_hex_str(text: &str) -> Result<Uint, ParseUintError> {
        let body = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X"));
        fold(body.unwrap_or(text), 16, |ch| ch.to_digit(16))
    }
}

/// Folds digits of `text` in radix `radix` using `digit` to classify characters.
fn fold(
    text: &str,
    radix: u64,
    digit: impl Fn(char) -> Option<u32>,
) -> Result<Uint, ParseUintError> {
    let mut acc = Uint::zero();
    let mut seen = false;
    for (index, ch) in text.char_indices() {
        if ch == '_' || ch.is_ascii_whitespace() {
            continue;
        }
        let value = digit(ch).ok_or(ParseUintError::InvalidDigit { ch, index })?;
        acc = acc.mul_u64(radix).add_u64(value as u64);
        seen = true;
    }
    if seen {
        Ok(acc)
    } else {
        Err(ParseUintError::Empty)
    }
}
