//! INTEGER decoding with DER minimality enforced.
//!
//! An INTEGER is returned as its raw big-endian two's-complement content octets
//! because a 2048-bit RSA modulus fits no Rust primitive. DER requires the
//! shortest encoding, so this module rejects:
//!
//! * empty content — every INTEGER has at least one octet;
//! * a leading `0x00` followed by an octet below `0x80` — the leading zero is
//!   only legal when it stops a positive value from looking negative;
//! * a leading `0xFF` followed by an octet with its top bit set — the mirror
//!   rule for negative values.
//!
//! This is a security property, not pedantry: if `0x02 0x02 0x00 0x05` and
//! `0x02 0x01 0x05` both parsed, one logical key would have two encodings and an
//! attacker could produce two distinct signatures over it.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::der::Reader;
//!
//! let five = [0x02, 0x01, 0x05];
//! assert_eq!(Reader::new(&five).read_integer_bytes().unwrap(), &[0x05]);
//!
//! // A positive value whose top bit is set keeps its legal leading zero.
//! let padded = [0x02, 0x02, 0x00, 0x80];
//! assert_eq!(Reader::new(&padded).read_integer_bytes().unwrap(), &[0x00, 0x80]);
//!
//! // An illegal leading zero is rejected.
//! let bad = [0x02, 0x02, 0x00, 0x05];
//! assert!(Reader::new(&bad).read_integer_bytes().is_err());
//! ```

use super::{der::Reader, error::Error, integer_rules, tag};

impl<'a> Reader<'a> {
    /// Read an INTEGER and return its raw big-endian content octets.
    ///
    /// # Returns
    ///
    /// The content octets, borrowed from the input. Two's-complement: a leading
    /// octet of `0x80` or above means the value is negative.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedTag`] for a non-INTEGER, and
    /// [`Error::MalformedValue`] for empty content or non-minimal padding.
    ///
    /// # Panics
    ///
    /// Never; the octets are inspected with `first`/`get`, not indexing.
    pub fn read_integer_bytes(&mut self) -> Result<&'a [u8], Error> {
        let tlv = self.read_expect(tag::INTEGER)?;
        integer_rules::check(tlv.content, tlv.offset)?;
        Ok(tlv.content)
    }

    /// Read an INTEGER expected to be a non-negative value fitting a `u64`.
    ///
    /// # Returns
    ///
    /// The value, with any single legal leading zero octet removed.
    ///
    /// # Errors
    ///
    /// As [`Reader::read_integer_bytes`], plus [`Error::MalformedValue`] when
    /// the value is negative or wider than eight significant octets.
    pub fn read_u64(&mut self) -> Result<u64, Error> {
        let tlv = self.read_expect(tag::INTEGER)?;
        integer_rules::check(tlv.content, tlv.offset)?;
        integer_rules::to_u64(tlv.content, tlv.offset)
    }
}
