//! `Display`, `LowerHex`, and `FromStr` for [`Uint`].
//!
//! `Display` is decimal, matching the primitive integer types. `FromStr` is
//! decimal-only and deliberately does *not* sniff a `0x` prefix, so that a
//! caller who means hex says so with `Uint::from_hex_str`.

use std::fmt;
use std::str::FromStr;

use super::error::ParseUintError;
use super::uint::Uint;

impl fmt::Display for Uint {
    /// Formats in decimal.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::from_u64(42).to_string(), "42");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_dec_string())
    }
}

impl fmt::LowerHex for Uint {
    /// Formats in lowercase hexadecimal without a prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(format!("{:x}", Uint::from_u64(255)), "ff");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex_string())
    }
}

impl FromStr for Uint {
    type Err = ParseUintError;

    /// Parses a decimal string.
    ///
    /// # Errors
    ///
    /// See [`Uint::from_dec_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// let value: Uint = "1234567890123456789012345".parse().unwrap();
    /// assert_eq!(value.to_string(), "1234567890123456789012345");
    /// ```
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Uint::from_dec_str(text)
    }
}
