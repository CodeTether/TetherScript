//! # Big-endian byte conversion (OS2IP / I2OSP)
//!
//! The external interface is big-endian **bytes** even though the internal
//! representation is little-endian **limbs** (see the `limbs` module), because
//! that is how PKCS#1 defines the two primitives this crate exists to serve:
//!
//! | PKCS#1 name | Here |
//! |---|---|
//! | OS2IP — octet string to integer | [`BigUint::from_be_bytes`] |
//! | I2OSP — integer to octet string, fixed length | [`BigUint::to_be_bytes`] |
//!
//! I2OSP is *fixed width*: an RSA signature or modulus is always exactly `k`
//! bytes, left-padded with zeros. So [`BigUint::to_be_bytes`] takes the width
//! and refuses a width narrower than the value with
//! [`BigUintError::WidthTooSmall`] rather than truncating — a silently truncated
//! modular exponentiation result would produce a signature check that
//! sometimes passes on the wrong input.
//!
//! Leading zero bytes on input are absorbed by normalization, so the round trip
//! is `from_be_bytes(b).to_be_bytes(b.len()) == b` for any input, including one
//! that is all zeros.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::{BigUint, BigUintError};
//!
//! let bytes = [0x00, 0x00, 0x01, 0x02, 0x03];
//! let value = BigUint::from_be_bytes(&bytes);
//! assert_eq!(value, BigUint::from_u64(0x01_02_03));
//! assert_eq!(value.to_be_bytes(5).unwrap(), bytes);
//! assert_eq!(value.byte_len(), 3);
//! assert_eq!(
//!     value.to_be_bytes(2),
//!     Err(BigUintError::WidthTooSmall { needed: 3, width: 2 })
//! );
//! ```

use super::error::BigUintError;
use super::limbs::BigUint;

impl BigUint {
    /// Decode a big-endian byte string (PKCS#1 OS2IP).
    ///
    /// # Arguments
    ///
    /// * `bytes` — the octets, most significant first. Leading zeros and an
    ///   empty slice are accepted; both normalize away.
    ///
    /// # Returns
    ///
    /// The normalized value. Never fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from_be_bytes(&[0x01, 0x00]), BigUint::from_u64(256));
    /// assert!(BigUint::from_be_bytes(&[]).is_zero());
    /// assert!(BigUint::from_be_bytes(&[0, 0, 0]).is_zero());
    /// ```
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        let mut limbs = Vec::with_capacity(bytes.len() / 8 + 1);
        let mut tail = bytes.len();
        while tail > 0 {
            let head = tail.saturating_sub(8);
            let mut limb = 0u64;
            for &byte in &bytes[head..tail] {
                limb = (limb << 8) | byte as u64;
            }
            limbs.push(limb);
            tail = head;
        }
        Self::normalized(limbs)
    }

    /// Minimum number of big-endian bytes that represent this value.
    ///
    /// # Returns
    ///
    /// `ceil(bit_len / 8)`, so `0` for zero. This is the smallest `width`
    /// [`BigUint::to_be_bytes`] will accept.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert_eq!(BigUint::zero().byte_len(), 0);
    /// assert_eq!(BigUint::from_u64(255).byte_len(), 1);
    /// assert_eq!(BigUint::from_u64(256).byte_len(), 2);
    /// ```
    pub fn byte_len(&self) -> usize {
        self.bit_len().div_ceil(8)
    }

    /// Encode as exactly `width` big-endian bytes (PKCS#1 I2OSP).
    ///
    /// # Arguments
    ///
    /// * `width` — the exact output length in bytes. The value is left-padded
    ///   with zeros to reach it, which is what RSA's fixed `k`-byte octet
    ///   strings require.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` of length `width`, most significant byte first.
    ///
    /// # Errors
    ///
    /// [`BigUintError::WidthTooSmall`] when `width < self.byte_len()`. The value
    /// is never truncated to fit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// let value = BigUint::from_u64(0xAB_CD);
    /// assert_eq!(value.to_be_bytes(4).unwrap(), vec![0x00, 0x00, 0xAB, 0xCD]);
    /// assert_eq!(value.to_be_bytes(2).unwrap(), vec![0xAB, 0xCD]);
    /// assert!(value.to_be_bytes(1).is_err());
    /// assert_eq!(BigUint::zero().to_be_bytes(3).unwrap(), vec![0, 0, 0]);
    /// ```
    pub fn to_be_bytes(&self, width: usize) -> Result<Vec<u8>, BigUintError> {
        let needed = self.byte_len();
        if width < needed {
            return Err(BigUintError::WidthTooSmall { needed, width });
        }
        let mut out = vec![0u8; width];
        for offset in 0..needed {
            let limb = self.limbs[offset / 8];
            out[width - 1 - offset] = (limb >> (8 * (offset % 8))) as u8;
        }
        Ok(out)
    }
}
