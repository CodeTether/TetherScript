//! The three simple primitives: OCTET STRING, NULL, and BOOLEAN.
//!
//! DER pins BOOLEAN to exactly one content octet, `0x00` for false and `0xFF`
//! for true; any other value is rejected rather than treated as truthy, because
//! accepting `0x01` would give `true` two encodings. NULL must have zero content
//! octets. OCTET STRING has no content constraint beyond its declared length,
//! which was already bounds-checked when the header was parsed.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::der::Reader;
//!
//! let octets = [0x04, 0x03, 0xde, 0xad, 0xbe];
//! assert_eq!(Reader::new(&octets).read_octet_string().unwrap(), &[0xde, 0xad, 0xbe]);
//!
//! let null = [0x05, 0x00];
//! assert!(Reader::new(&null).read_null().is_ok());
//!
//! let yes = [0x01, 0x01, 0xff];
//! assert!(Reader::new(&yes).read_bool().unwrap());
//!
//! // 0x01 is a legal BER "true" but not legal DER.
//! let ber = [0x01, 0x01, 0x01];
//! assert!(Reader::new(&ber).read_bool().is_err());
//! ```

use super::{der::Reader, error::Error, tag};

impl<'a> Reader<'a> {
    /// Read an OCTET STRING and return its content octets.
    ///
    /// # Returns
    ///
    /// The content octets, borrowed from the input.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedTag`] when the next value is not an OCTET STRING.
    pub fn read_octet_string(&mut self) -> Result<&'a [u8], Error> {
        Ok(self.read_expect(tag::OCTET_STRING)?.content)
    }

    /// Read a NULL value.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedTag`] for a non-NULL tag, or
    /// [`Error::MalformedValue`] if it carries any content octets.
    pub fn read_null(&mut self) -> Result<(), Error> {
        let tlv = self.read_expect(tag::NULL)?;
        if !tlv.content.is_empty() {
            return Err(Error::MalformedValue {
                offset: tlv.offset,
                tag: tag::NULL,
                reason: "NULL must have zero content octets",
            });
        }
        Ok(())
    }

    /// Read a BOOLEAN.
    ///
    /// # Returns
    ///
    /// `false` for content `0x00`, `true` for content `0xFF`.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedTag`] for a non-BOOLEAN, or
    /// [`Error::MalformedValue`] for any content other than one `0x00`/`0xFF`.
    ///
    /// # Panics
    ///
    /// Never; the content is matched as a slice pattern.
    pub fn read_bool(&mut self) -> Result<bool, Error> {
        let tlv = self.read_expect(tag::BOOLEAN)?;
        match tlv.content {
            [0x00] => Ok(false),
            [0xff] => Ok(true),
            _ => Err(Error::MalformedValue {
                offset: tlv.offset,
                tag: tag::BOOLEAN,
                reason: "BOOLEAN must be one octet, 0x00 or 0xFF",
            }),
        }
    }
}
