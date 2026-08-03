//! BIT STRING decoding, preserving the leading unused-bits octet.
//!
//! A primitive BIT STRING's content is one "unused bits" octet in `0..=7`
//! followed by the value octets. An RSA `SubjectPublicKeyInfo` wraps its
//! PKCS#1 `RSAPublicKey` in a BIT STRING with zero unused bits, and callers
//! must be able to see that count rather than have it silently dropped.
//!
//! DER also requires that unused bits be zero, and that an empty bit string
//! declare zero unused bits; both are enforced.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::der::Reader;
//!
//! // BIT STRING with 0 unused bits wrapping 0x30 0x00 (an empty SEQUENCE).
//! let der = [0x03, 0x03, 0x00, 0x30, 0x00];
//! let bits = Reader::new(&der).read_bit_string().unwrap();
//! assert_eq!(bits.unused_bits, 0);
//! assert_eq!(bits.bytes, &[0x30, 0x00]);
//! ```

use super::{der::Reader, error::Error, tag};

/// A decoded BIT STRING: its value octets plus its unused-bit count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitString<'a> {
    /// Number of unused low-order bits in the final byte, always `0..=7`.
    pub unused_bits: u8,
    /// The value octets, excluding the unused-bits prefix octet.
    pub bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    /// Read a BIT STRING.
    ///
    /// # Returns
    ///
    /// A [`BitString`] borrowing the value octets from the input.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedTag`] for a non-BIT-STRING, and
    /// [`Error::MalformedValue`] for empty content, an unused-bit count above 7,
    /// a non-zero count on empty value octets, or non-zero unused bits.
    ///
    /// # Panics
    ///
    /// Never; the content is destructured by slice pattern, not indexed.
    pub fn read_bit_string(&mut self) -> Result<BitString<'a>, Error> {
        let tlv = self.read_expect(tag::BIT_STRING)?;
        super::bit_string_rules::decode(tlv.content, tlv.offset)
    }
}
