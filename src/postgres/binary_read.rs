//! # Checked cursor over a binary field body
//!
//! Bounds-checking front door for every binary decoder. A field body arrives from
//! a network peer, so a length is a *claim*, not a fact: [`Reader::take`]
//! validates before slicing and reports [`DecodeError::Truncated`] naming the
//! sub-field instead of panicking. Nothing in this module indexes or slices with
//! an unchecked length, and nothing calls `unwrap` on untrusted data.
//!
//! Numeric accessors live in the sibling `binary_read_int.rs` and
//! `binary_read_float.rs` so bounds checking and integer/float widths stay one
//! concern each. All of them read **big-endian**, because the PostgreSQL wire
//! protocol is network byte order throughout.

use super::error::DecodeError;

#[path = "binary_read_float.rs"]
mod float;
#[path = "binary_read_int.rs"]
mod int;

/// A checked position within a binary field body.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::Reader;
///
/// let mut reader = Reader::new(&[0x00, 0x2A]);
/// assert_eq!(reader.i16("int2").unwrap(), 42);
/// assert!(reader.finish("int2").is_ok());
///
/// // Truncation is a named error, never a panic.
/// let mut short = Reader::new(&[0x00]);
/// assert!(short.i16("int2").is_err());
/// ```
pub struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Wrap a field body.
    ///
    /// # Arguments
    ///
    /// * `bytes` — the field body, already separated from its length prefix.
    ///
    /// # Returns
    ///
    /// A reader positioned at byte zero.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    /// Bytes not yet consumed.
    ///
    /// # Returns
    ///
    /// The remaining length, saturating at zero so it can never underflow.
    pub fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    /// Consume exactly `len` bytes.
    ///
    /// # Arguments
    ///
    /// * `what` — name used in the error message, e.g. `"int4"`.
    /// * `len` — bytes the layout requires.
    ///
    /// # Returns
    ///
    /// The consumed slice, with the cursor advanced past it.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Truncated`] when fewer than `len` bytes remain. The offset
    /// addition is checked, so a hostile length cannot wrap around and slip past
    /// the bounds test.
    pub fn take(&mut self, what: &'static str, len: usize) -> Result<&'a [u8], DecodeError> {
        let short = DecodeError::Truncated {
            what,
            need: len,
            have: self.remaining(),
        };
        let end = self.pos.checked_add(len).ok_or_else(|| short.clone())?;
        let slice = self.bytes.get(self.pos..end).ok_or(short)?;
        self.pos = end;
        Ok(slice)
    }

    /// Consume everything left, which may legitimately be empty.
    ///
    /// # Returns
    ///
    /// The tail slice. Used by `text`, `bytea`, `json`, and `jsonb`, whose bodies
    /// run to the end of the field.
    pub fn rest(&mut self) -> &'a [u8] {
        let slice = self.bytes.get(self.pos..).unwrap_or(&[]);
        self.pos = self.bytes.len();
        slice
    }

    /// Assert the layout consumed the whole body.
    ///
    /// # Arguments
    ///
    /// * `what` — field name for the error message.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Overlong`] when trailing bytes remain, which means the field
    /// was not the type its OID claimed. Ignoring the tail would let a mismatched
    /// OID decode to a plausible wrong value.
    pub fn finish(&self, what: &'static str) -> Result<(), DecodeError> {
        if self.pos == self.bytes.len() {
            return Ok(());
        }
        Err(DecodeError::Overlong {
            what,
            expected: self.pos,
            got: self.bytes.len(),
        })
    }
}
