//! OBJECT IDENTIFIER decoding to dotted-decimal notation.
//!
//! The content octets are a sequence of base-128 subidentifiers, most
//! significant group first, with the top bit of every octet but the last of a
//! group set as a continuation flag. The first subidentifier packs the first two
//! arcs as `40 * arc1 + arc2`, which is why `1.2.840.113549.1.1.1`
//! (`rsaEncryption`) begins with the single octet `0x2A`.
//!
//! DER minimality is enforced: a subidentifier may not begin with `0x80`, and
//! the content may not end mid-group.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::{der::Reader, oid};
//!
//! // OID 1.2.840.113549.1.1.1 — PKCS#1 rsaEncryption.
//! let der = [
//!     0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01,
//! ];
//! assert_eq!(Reader::new(&der).read_oid().unwrap(), oid::RSA_ENCRYPTION);
//! ```

use super::{der::Reader, error::Error, oid_decode, tag};

/// Dotted-decimal OID of PKCS#1 `rsaEncryption`.
pub const RSA_ENCRYPTION: &str = "1.2.840.113549.1.1.1";

impl Reader<'_> {
    /// Read an OBJECT IDENTIFIER and render it as dotted decimal.
    ///
    /// # Returns
    ///
    /// An owned `String` such as `"1.2.840.113549.1.1.1"`.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedTag`] for a non-OID, and [`Error::MalformedValue`] for
    /// empty content, a truncated final subidentifier, a non-minimal `0x80`
    /// prefix, or an arc too large for a `u64`.
    ///
    /// # Panics
    ///
    /// Never; every octet is fetched with `slice::get`.
    pub fn read_oid(&mut self) -> Result<String, Error> {
        let tlv = self.read_expect(tag::OBJECT_IDENTIFIER)?;
        oid_decode::to_dotted(tlv.content, tlv.offset)
    }
}
