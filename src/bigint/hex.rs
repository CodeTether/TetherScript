//! # Hexadecimal parsing and rendering
//!
//! A convenience surface for the places where big integers are written down by
//! humans: RSA test vectors, JWKS fixtures, and error messages. Not on any hot
//! path.
//!
//! Hex is big-endian like the `bytes` module, and an odd digit count is accepted
//! by treating the leading digit as a half byte, so `"abc"` is `0xabc`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUint;
//!
//! let value = BigUint::from_hex("10001").unwrap();
//! assert_eq!(value, BigUint::from_u64(65_537));
//! assert_eq!(value.to_hex(), "10001");
//! assert_eq!(BigUint::zero().to_hex(), "0");
//! assert!(BigUint::from_hex("12zz").is_none());
//! ```

use super::limbs::BigUint;

impl BigUint {
    /// Parse a big-endian hexadecimal string.
    ///
    /// # Arguments
    ///
    /// * `text` — hex digits, either case, with an optional `0x` prefix and
    ///   optional `_` separators. An odd digit count is allowed.
    ///
    /// # Returns
    ///
    /// `Some(value)` on success, or `None` when `text` holds a character that is
    /// not a hex digit, `_`, or the prefix. An empty digit string is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from_hex("0xFF").unwrap(), BigUint::from_u64(255));
    /// assert_eq!(BigUint::from_hex("ff_ff").unwrap(), BigUint::from_u64(65_535));
    /// assert!(BigUint::from_hex("0x").unwrap().is_zero());
    /// assert!(BigUint::from_hex("g").is_none());
    /// ```
    pub fn from_hex(text: &str) -> Option<Self> {
        let body = text
            .strip_prefix("0x")
            .or_else(|| text.strip_prefix("0X"))
            .unwrap_or(text);
        let mut value = Self::zero();
        let sixteen = Self::from_u64(16);
        for character in body.chars().filter(|c| *c != '_') {
            let digit = character.to_digit(16)?;
            value = value.mul(&sixteen).add(&Self::from_u64(u64::from(digit)));
        }
        Some(value)
    }

    /// Render as lowercase big-endian hex with no leading zeros.
    ///
    /// # Returns
    ///
    /// The digits, or `"0"` for zero. Round-trips through
    /// [`BigUint::from_hex`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from_u64(255).to_hex(), "ff");
    /// assert_eq!(BigUint::from_limbs_le(vec![0, 1]).to_hex(), "10000000000000000");
    /// ```
    pub fn to_hex(&self) -> String {
        let Some((top, rest)) = self.limbs.split_last() else {
            return "0".to_string();
        };
        let mut out = format!("{top:x}");
        for limb in rest.iter().rev() {
            out.push_str(&format!("{limb:016x}"));
        }
        out
    }
}
