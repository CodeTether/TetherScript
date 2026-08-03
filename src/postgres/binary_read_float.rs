//! # Big-endian IEEE-754 accessors for [`Reader`]
//!
//! `float4` and `float8` are sent as the raw IEEE-754 single and double bit
//! patterns, **big-endian**, so the bytes are assembled with `from_be_bytes` and
//! then reinterpreted with `f32::from_bits` / `f64::from_bits`.
//!
//! Reinterpreting bits is exact and lossless: no arithmetic happens here, so a
//! NaN, an infinity, or a subnormal survives the trip unchanged. `float4` widens
//! to `f64` because tetherscript has a single `Float` type, and that widening is
//! exact — every `f32` is representable as an `f64`.
//!
//! Note that `float4`/`float8` are *not* how money should be stored; see the
//! `numeric` decoder for why a decimal never goes through these functions.

use super::super::error::DecodeError;
use super::Reader;

impl Reader<'_> {
    /// Read a big-endian `float4` and widen it exactly to `f64`.
    ///
    /// # Arguments
    ///
    /// * `what` — field name for the error message.
    ///
    /// # Returns
    ///
    /// The single-precision value widened to `f64`, cursor advanced 4 bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Truncated`] when fewer than 4 bytes remain.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::postgres::binary::Reader;
    ///
    /// // 1.5f32 has the bit pattern 0x3FC00000.
    /// let mut reader = Reader::new(&[0x3F, 0xC0, 0x00, 0x00]);
    /// assert_eq!(reader.f32("float4").unwrap(), 1.5);
    /// ```
    pub fn f32(&mut self, what: &'static str) -> Result<f64, DecodeError> {
        let b = self.take(what, 4)?;
        let bits = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        Ok(f32::from_bits(bits) as f64)
    }

    /// Read a big-endian `float8`.
    ///
    /// # Arguments
    ///
    /// * `what` — field name for the error message.
    ///
    /// # Returns
    ///
    /// The double-precision value, cursor advanced 8 bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Truncated`] when fewer than 8 bytes remain.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::postgres::binary::Reader;
    ///
    /// // -2.25f64 has the bit pattern 0xC002000000000000.
    /// let mut reader = Reader::new(&[0xC0, 0x02, 0, 0, 0, 0, 0, 0]);
    /// assert_eq!(reader.f64("float8").unwrap(), -2.25);
    /// ```
    pub fn f64(&mut self, what: &'static str) -> Result<f64, DecodeError> {
        let b = self.take(what, 8)?;
        let wide = [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]];
        Ok(f64::from_bits(u64::from_be_bytes(wide)))
    }
}
