//! Typed DER/ASN.1 decoding errors.
//!
//! Every variant carries the byte `offset` at which the problem was detected,
//! measured from the start of the *original* document even when the failure
//! occurs inside a nested SEQUENCE. That makes failures reproducible against a
//! hex dump: the offset always names a real byte the caller handed us.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::error::Error;
//!
//! // A truncated SEQUENCE header: tag 0x30 with no length octet.
//! let err = Error::UnexpectedEnd { offset: 1 };
//! assert_eq!(err.offset(), 1);
//! assert!(err.to_string().contains("offset 1"));
//! ```

/// A DER decoding failure.
///
/// Variants are grouped by cause: structural (`UnexpectedEnd`,
/// `LengthExceedsInput`), encoding-rule violations (`IndefiniteLength`,
/// `NonMinimalLength`, `ReservedLength`), resource limits (`LengthTooLarge`,
/// `DepthExceeded`), and content-level problems (`MalformedValue`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Input ended while a tag or length octet was still required.
    UnexpectedEnd { offset: usize },
    /// A definite length named more content octets than the input holds.
    LengthExceedsInput {
        offset: usize,
        length: usize,
        available: usize,
    },
    /// Indefinite length (`0x80`) is forbidden in DER.
    IndefiniteLength { offset: usize },
    /// A long-form length that should have used a shorter encoding.
    NonMinimalLength { offset: usize },
    /// The reserved length octet `0xFF`.
    ReservedLength { offset: usize },
    /// A length field too wide to address on this platform.
    LengthTooLarge { offset: usize, bytes: usize },
    /// High-tag-number (multi-byte) identifiers are not supported.
    HighTagNumber { offset: usize },
    /// The tag found is not the tag the caller asked for.
    UnexpectedTag {
        offset: usize,
        expected: u8,
        found: u8,
    },
    /// Nesting exceeded [`crate::asn1::der::MAX_DEPTH`].
    DepthExceeded { offset: usize, max_depth: usize },
    /// A value's content octets violate the rules for its tag.
    MalformedValue {
        offset: usize,
        tag: u8,
        reason: &'static str,
    },
    /// Unconsumed bytes remain where the structure should have ended.
    TrailingData { offset: usize },
    /// PEM armour or base64 body could not be decoded.
    Pem { offset: usize, reason: String },
}

impl Error {
    /// Byte offset, in the original input, at which the error was detected.
    ///
    /// # Returns
    ///
    /// The offset recorded in the variant. Never panics and never allocates.
    pub fn offset(&self) -> usize {
        match self {
            Error::UnexpectedEnd { offset, .. }
            | Error::LengthExceedsInput { offset, .. }
            | Error::IndefiniteLength { offset, .. }
            | Error::NonMinimalLength { offset, .. }
            | Error::ReservedLength { offset, .. }
            | Error::LengthTooLarge { offset, .. }
            | Error::HighTagNumber { offset, .. }
            | Error::UnexpectedTag { offset, .. }
            | Error::DepthExceeded { offset, .. }
            | Error::MalformedValue { offset, .. }
            | Error::TrailingData { offset, .. }
            | Error::Pem { offset, .. } => *offset,
        }
    }
}
