//! Core TLV cursor operations on [`Reader`]: read one value, require a specific
//! tag, and assert the cursor is exhausted.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::{der::Reader, tag};
//!
//! let mut reader = Reader::new(&[0x05, 0x00]);
//! let tlv = reader.read_expect(tag::NULL).unwrap();
//! assert!(tlv.content.is_empty());
//! reader.finish().unwrap();
//! ```

use super::{der::Reader, error::Error, header, tlv::Tlv};

impl<'a> Reader<'a> {
    /// Read the next TLV and advance past it.
    ///
    /// # Returns
    ///
    /// The decoded [`Tlv`], whose `content` borrows the reader's input.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedEnd`] when no bytes remain, plus any tag or length
    /// error reported by the header parser.
    ///
    /// # Panics
    ///
    /// Never; all reads are bounds-checked.
    pub fn read_tlv(&mut self) -> Result<Tlv<'a>, Error> {
        if self.is_empty() {
            return Err(Error::UnexpectedEnd { offset: self.offset() });
        }
        let (tlv, next) = header::parse(self.input, self.pos, self.base)?;
        self.pos = next;
        Ok(tlv)
    }

    /// Read the next TLV, requiring a specific identifier octet.
    ///
    /// # Arguments
    ///
    /// * `expected` — the identifier octet the value must carry.
    ///
    /// # Returns
    ///
    /// The decoded [`Tlv`]. The cursor does not advance on a tag mismatch.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedTag`] naming both tags, or any error from
    /// [`Reader::read_tlv`].
    pub fn read_expect(&mut self, expected: u8) -> Result<Tlv<'a>, Error> {
        let saved = self.pos;
        let tlv = self.read_tlv()?;
        if tlv.tag != expected {
            self.pos = saved;
            return Err(Error::UnexpectedTag { offset: tlv.offset, expected, found: tlv.tag });
        }
        Ok(tlv)
    }

    /// Require that the reader is exhausted.
    ///
    /// # Errors
    ///
    /// [`Error::TrailingData`] naming the offset of the first leftover byte.
    pub fn finish(&self) -> Result<(), Error> {
        if self.is_empty() {
            return Ok(());
        }
        Err(Error::TrailingData { offset: self.offset() })
    }
}
