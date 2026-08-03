//! One decoded tag-length-value triple, borrowed from the input buffer.
//!
//! A [`Tlv`] never owns bytes: `content` is a subslice of the buffer handed to
//! [`Reader::new`](super::der::Reader::new), which was bounds-checked before the
//! slice was created. `offset` and `content_offset` are absolute offsets in the
//! original document, so they stay meaningful inside nested SEQUENCEs.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::{der::Reader, tag};
//!
//! // NULL inside a SEQUENCE: 30 02 05 00
//! let der = [0x30, 0x02, 0x05, 0x00];
//! let mut outer = Reader::new(&der);
//! let seq = outer.read_tlv().unwrap();
//! assert_eq!(seq.tag, tag::SEQUENCE);
//! assert_eq!(seq.offset, 0);
//! assert_eq!(seq.content_offset, 2);
//! assert_eq!(seq.content, &[0x05, 0x00]);
//! assert_eq!(seq.end_offset(), 4);
//! assert!(seq.is_constructed());
//! ```

/// A tag-length-value triple with absolute offsets attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tlv<'a> {
    /// The single identifier octet, e.g. `0x30` for SEQUENCE.
    pub tag: u8,
    /// Absolute offset of the identifier octet in the original document.
    pub offset: usize,
    /// Absolute offset of the first content octet.
    pub content_offset: usize,
    /// The content octets, already bounds-checked against the input.
    pub content: &'a [u8],
}

impl Tlv<'_> {
    /// Absolute offset one past the final content octet.
    ///
    /// # Returns
    ///
    /// `content_offset + content.len()`, saturating instead of overflowing.
    pub fn end_offset(&self) -> usize {
        self.content_offset.saturating_add(self.content.len())
    }

    /// Report whether this value is constructed (its content is more TLVs).
    ///
    /// # Returns
    ///
    /// `true` for SEQUENCE, SET, and other constructed tags.
    pub fn is_constructed(&self) -> bool {
        super::tag::is_constructed(self.tag)
    }
}
