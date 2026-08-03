//! Descending into constructed values (SEQUENCE, SET) without recursion.
//!
//! [`Reader::read_sequence`] returns a *child* reader borrowing the parent's
//! content slice with `depth + 1`. Nothing recurses inside the decoder, so the
//! only stack growth is whatever the caller's own traversal loop creates. The
//! child inherits an absolute `base`, so offsets reported from deep inside a
//! structure still name real bytes of the original document.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::der::Reader;
//!
//! // SEQUENCE { SEQUENCE { NULL } }
//! let der = [0x30, 0x04, 0x30, 0x02, 0x05, 0x00];
//! let mut top = Reader::new(&der);
//! let mut outer = top.read_sequence().unwrap();
//! assert_eq!(outer.depth(), 1);
//! let mut inner = outer.read_sequence().unwrap();
//! assert_eq!(inner.depth(), 2);
//! assert_eq!(inner.offset(), 4);
//! inner.read_null().unwrap();
//! inner.finish().unwrap();
//! ```

use super::{
    der::{Reader, MAX_DEPTH},
    error::Error,
    tag,
    tlv::Tlv,
};

impl<'a> Reader<'a> {
    /// Read a SEQUENCE and return a reader over its contents.
    ///
    /// # Returns
    ///
    /// A child [`Reader`] positioned at the first element, with `depth` one
    /// greater than this reader's.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedTag`] if the next value is not a SEQUENCE, and
    /// [`Error::DepthExceeded`] once nesting would pass [`MAX_DEPTH`].
    pub fn read_sequence(&mut self) -> Result<Reader<'a>, Error> {
        let tlv = self.read_expect(tag::SEQUENCE)?;
        self.descend(tlv)
    }

    /// Read a SET and return a reader over its contents.
    ///
    /// # Returns
    ///
    /// A child [`Reader`] over the SET's contents.
    ///
    /// # Errors
    ///
    /// As [`Reader::read_sequence`], but requiring the SET tag.
    pub fn read_set(&mut self) -> Result<Reader<'a>, Error> {
        let tlv = self.read_expect(tag::SET)?;
        self.descend(tlv)
    }

    /// Build a child reader over an already-read constructed value.
    ///
    /// # Arguments
    ///
    /// * `tlv` — a constructed TLV previously read from this reader.
    ///
    /// # Returns
    ///
    /// A child reader over `tlv.content`.
    ///
    /// # Errors
    ///
    /// [`Error::DepthExceeded`] when the child's depth would exceed
    /// [`MAX_DEPTH`], and [`Error::MalformedValue`] if `tlv` is primitive.
    pub fn descend(&self, tlv: Tlv<'a>) -> Result<Reader<'a>, Error> {
        if !tlv.is_constructed() {
            return Err(Error::MalformedValue {
                offset: tlv.offset,
                tag: tlv.tag,
                reason: "cannot descend into a primitive value",
            });
        }
        let depth = self.depth.saturating_add(1);
        if depth > MAX_DEPTH {
            return Err(Error::DepthExceeded {
                offset: tlv.offset,
                max_depth: MAX_DEPTH,
            });
        }
        Ok(Reader {
            input: tlv.content,
            pos: 0,
            base: tlv.content_offset,
            depth,
        })
    }
}
