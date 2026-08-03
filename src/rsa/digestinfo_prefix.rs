//! # DER `DigestInfo` prefix tables
//!
//! One responsibility: hold the exact octet strings that precede the message
//! digest inside an EMSA-PKCS1-v1_5 encoded message.
//!
//! ## Provenance
//!
//! These byte strings are quoted verbatim from **RFC 8017 section 9.2, Notes**
//! ("PKCS #1 v1.5 signature scheme", note 1), which lists the full DER encoding
//! of
//!
//! ```text
//! DigestInfo ::= SEQUENCE {
//!     digestAlgorithm AlgorithmIdentifier,
//!     digest          OCTET STRING
//! }
//! ```
//!
//! for each hash function, with the `parameters` field present and set to
//! `NULL` (`05 00`). RFC 8017 is explicit that this fixed prefix form is what
//! implementations must produce and accept, so this module treats the bytes as
//! an opaque constant rather than running a general DER parser.
//!
//! ## Why constants and not a DER parser
//!
//! A permissive parser is exactly how Bleichenbacher-style PKCS#1 v1.5 forgeries
//! get in: accepting non-minimal lengths, an absent or differently encoded
//! `NULL`, or trailing fields inside the `SEQUENCE` all widen the set of
//! integers that "decode" to a valid signature. Comparing against a frozen byte
//! table admits exactly one encoding per algorithm.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::rsa::DigestAlgorithm;
//!
//! // SHA-256: SEQUENCE(0x30) len 0x31, then the 2.16.840.1.101.3.4.2.1 OID.
//! assert_eq!(DigestAlgorithm::Sha256.der_prefix()[0], 0x30);
//! assert_eq!(DigestAlgorithm::Sha256.der_prefix().len(), 19);
//! ```

/// DER `DigestInfo` prefix for SHA-1 (OID 1.3.14.3.2.26), 15 octets.
pub(super) const SHA1: &[u8] = &[
    0x30, 0x21, 0x30, 0x09, 0x06, 0x05, 0x2b, 0x0e, 0x03, 0x02, 0x1a, 0x05, 0x00, 0x04, 0x14,
];

/// DER `DigestInfo` prefix for SHA-256 (OID 2.16.840.1.101.3.4.2.1), 19 octets.
pub(super) const SHA256: &[u8] = &[
    0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01, 0x05,
    0x00, 0x04, 0x20,
];

/// DER `DigestInfo` prefix for SHA-384 (OID 2.16.840.1.101.3.4.2.2), 19 octets.
pub(super) const SHA384: &[u8] = &[
    0x30, 0x41, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x02, 0x05,
    0x00, 0x04, 0x30,
];

/// DER `DigestInfo` prefix for SHA-512 (OID 2.16.840.1.101.3.4.2.3), 19 octets.
pub(super) const SHA512: &[u8] = &[
    0x30, 0x51, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x03, 0x05,
    0x00, 0x04, 0x40,
];
