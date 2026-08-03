//! # Big-endian integer accessors for [`Reader`]
//!
//! Separated from the cursor itself so bounds checking and integer widths stay
//! one concern each.
//!
//! **Every read is `from_be_bytes`.** PostgreSQL sends network byte order, and on
//! a little-endian host a `from_le_bytes` slip does not fail loudly — it yields a
//! byte-swapped number, so `0x00010203` reads as `50462976` instead of `66051`.
//! This file and its `float` sibling are the only places in the module where
//! integers are assembled from bytes, which is what makes that auditable.

use super::super::error::DecodeError;
use super::Reader;

impl Reader<'_> {
    /// Read a big-endian `i16` (`int2`, and every protocol count field).
    ///
    /// # Arguments
    ///
    /// * `what` — field name for the error message.
    ///
    /// # Returns
    ///
    /// The value, with the cursor advanced 2 bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Truncated`] when fewer than 2 bytes remain.
    pub fn i16(&mut self, what: &'static str) -> Result<i16, DecodeError> {
        let b = self.take(what, 2)?;
        Ok(i16::from_be_bytes([b[0], b[1]]))
    }

    /// Read a big-endian `u16`, used for the `numeric` sign word.
    ///
    /// # Arguments
    ///
    /// * `what` — field name for the error message.
    ///
    /// # Returns
    ///
    /// The value, with the cursor advanced 2 bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Truncated`] when fewer than 2 bytes remain.
    pub fn u16(&mut self, what: &'static str) -> Result<u16, DecodeError> {
        let b = self.take(what, 2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    /// Read a big-endian `i32` (`int4`, `date`, and every array header word).
    ///
    /// # Arguments
    ///
    /// * `what` — field name for the error message.
    ///
    /// # Returns
    ///
    /// The value, with the cursor advanced 4 bytes.
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
    /// // Big-endian 0x00010203 is 66051, not the byte-swapped 50462976.
    /// let mut reader = Reader::new(&[0, 1, 2, 3]);
    /// assert_eq!(reader.i32("int4").unwrap(), 66_051);
    /// ```
    pub fn i32(&mut self, what: &'static str) -> Result<i32, DecodeError> {
        let b = self.take(what, 4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Read a big-endian `i64` (`int8`, `time`, `timestamp`, `timestamptz`).
    ///
    /// # Arguments
    ///
    /// * `what` — field name for the error message.
    ///
    /// # Returns
    ///
    /// The value, with the cursor advanced 8 bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Truncated`] when fewer than 8 bytes remain.
    pub fn i64(&mut self, what: &'static str) -> Result<i64, DecodeError> {
        let b = self.take(what, 8)?;
        let wide = [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]];
        Ok(i64::from_be_bytes(wide))
    }
}
