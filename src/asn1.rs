//! DER/ASN.1 decoding for cryptographic key material.
//!
//! Exists so an RS256 JWT can be verified: an RSA public key arrives as PEM or DER, and
//! nothing in-tree could read either. Integer components come back as raw big-endian byte
//! slices, deliberately decoupled from [`crate::bignum`] so each layer stays reviewable on
//! its own.
//!
//! # Layers
//!
//! | Concern | Modules |
//! |---|---|
//! | Tag, length, value | `tag`, `length*`, `header`, `tlv`, `primitive` |
//! | Reader and nesting | `der`, `reader_core`, `reader_nested` |
//! | Typed content | `integer*`, `bit_string*`, `oid*` |
//! | PEM armour | `pem*` |
//! | Errors | `error*` |
//!
//! # Security posture
//!
//! This parses untrusted input, so strictness is the point rather than a nicety. A
//! declared length longer than the buffer is refused instead of trusted for a slice;
//! indefinite-form lengths are refused because DER forbids them and accepting them invites
//! parser-differential attacks; a long-form length that could have been short is refused
//! because tolerating both spellings is how signature-verification bypasses happen; and
//! nesting is bounded because a deeply nested SEQUENCE is a stack-exhaustion vector.
//!
//! PEM bodies are decoded with [`crate::system`]'s standard-alphabet base64, which already
//! skips CR, LF, tab, and space. The base64url decoders used by JWT and CSRF reject `+`,
//! `/`, and `=` by design and would be wrong here.

#[path = "asn1/bit_string.rs"]
pub mod bit_string;
#[path = "asn1/bit_string_rules.rs"]
mod bit_string_rules;
#[path = "asn1/der.rs"]
pub mod der;
#[path = "asn1/error.rs"]
pub mod error;
#[path = "asn1/error_display.rs"]
mod error_display;
#[path = "asn1/error_text_content.rs"]
mod error_text_content;
#[path = "asn1/error_text_length.rs"]
mod error_text_length;
#[path = "asn1/error_text_structure.rs"]
mod error_text_structure;
#[path = "asn1/header.rs"]
pub mod header;
#[path = "asn1/integer.rs"]
pub mod integer;
#[path = "asn1/integer_rules.rs"]
mod integer_rules;
#[path = "asn1/length.rs"]
pub mod length;
#[path = "asn1/length_long.rs"]
mod length_long;
#[path = "asn1/oid.rs"]
pub mod oid;
#[path = "asn1/oid_decode.rs"]
mod oid_decode;
#[path = "asn1/oid_subid.rs"]
mod oid_subid;
#[path = "asn1/pem.rs"]
pub mod pem;
#[path = "asn1/pem_armour.rs"]
mod pem_armour;
#[path = "asn1/pem_body.rs"]
mod pem_body;
#[path = "asn1/primitive.rs"]
pub mod primitive;
#[path = "asn1/reader_core.rs"]
mod reader_core;
#[path = "asn1/reader_nested.rs"]
mod reader_nested;
#[path = "asn1/tag.rs"]
pub mod tag;
#[path = "asn1/tlv.rs"]
pub mod tlv;
