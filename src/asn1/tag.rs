//! Universal-class ASN.1 tag octets used by the X.509 / PKCS#1 structures this
//! decoder targets.
//!
//! Only the low-tag-number form is modelled: a single identifier octet whose
//! low five bits hold the tag number. That covers every tag in a
//! `SubjectPublicKeyInfo`, and anything else is rejected loudly by
//! [`crate::asn1::der::Reader`] rather than guessed at.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::tag;
//!
//! assert_eq!(tag::SEQUENCE, 0x30);
//! assert!(tag::is_constructed(tag::SEQUENCE));
//! assert!(!tag::is_constructed(tag::INTEGER));
//! ```

/// `BOOLEAN`, universal tag 1.
pub const BOOLEAN: u8 = 0x01;
/// `INTEGER`, universal tag 2.
pub const INTEGER: u8 = 0x02;
/// `BIT STRING`, universal tag 3.
pub const BIT_STRING: u8 = 0x03;
/// `OCTET STRING`, universal tag 4.
pub const OCTET_STRING: u8 = 0x04;
/// `NULL`, universal tag 5.
pub const NULL: u8 = 0x05;
/// `OBJECT IDENTIFIER`, universal tag 6.
pub const OBJECT_IDENTIFIER: u8 = 0x06;
/// `SEQUENCE` / `SEQUENCE OF`, universal tag 16, constructed.
pub const SEQUENCE: u8 = 0x30;
/// `SET` / `SET OF`, universal tag 17, constructed.
pub const SET: u8 = 0x31;

/// Mask selecting the tag number out of an identifier octet.
pub const TAG_NUMBER_MASK: u8 = 0x1f;
/// Bit 6 of an identifier octet: set when the value is constructed.
pub const CONSTRUCTED_BIT: u8 = 0x20;

/// Report whether an identifier octet describes a constructed value.
///
/// # Arguments
///
/// * `tag` — a single identifier octet.
///
/// # Returns
///
/// `true` when bit 6 is set, meaning the contents are themselves TLVs.
pub fn is_constructed(tag: u8) -> bool {
    tag & CONSTRUCTED_BIT != 0
}

/// Report whether an identifier octet uses the unsupported high-tag-number form.
///
/// # Arguments
///
/// * `tag` — a single identifier octet.
///
/// # Returns
///
/// `true` when the tag number bits are all ones, which means the real tag
/// number continues into following octets.
pub fn is_high_tag_number(tag: u8) -> bool {
    tag & TAG_NUMBER_MASK == TAG_NUMBER_MASK
}
