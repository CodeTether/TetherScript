//! RSASSA-PKCS1-v1_5 signature verification.
//!
//! The last piece of the RS256 path: a JWT signed with RSA is verified by applying the
//! public-key operation to the signature and checking the recovered block byte for byte.
//!
//! # Layers
//!
//! | Concern | Modules |
//! |---|---|
//! | Verification entry | `verify` |
//! | Encoded-message block | `pkcs1`, `pkcs1_padding` |
//! | DigestInfo | `digestinfo*` |
//! | Public key checks | `key*` |
//! | Constant-time compare | `ct` |
//! | Errors | `error*` |
//!
//! # Security posture
//!
//! Verification **encodes the expected block and compares**, rather than parsing the
//! recovered block and trusting what it finds. A lenient parser here accepts forged
//! signatures — the Bleichenbacher 2006 attack class, which broke real libraries — so the
//! padding is checked in full: the leading `0x00 0x01`, every `0xFF` byte, at least eight
//! of them, and the `0x00` separator.
//!
//! This verifies only. It does not sign, and it must never be used for decryption, where
//! PKCS#1 v1.5 has its own padding-oracle problems.

#[path = "rsa/ct.rs"]
pub mod ct;
#[cfg(test)]
#[path = "rsa/ct_tests.rs"]
mod ct_tests;
#[path = "rsa/digestinfo.rs"]
pub mod digestinfo;
#[path = "rsa/digestinfo_prefix.rs"]
mod digestinfo_prefix;
#[cfg(test)]
#[path = "rsa/digestinfo_tests.rs"]
mod digestinfo_tests;
#[path = "rsa/error.rs"]
pub mod error;
#[path = "rsa/error_display.rs"]
mod error_display;
#[path = "rsa/error_text_key.rs"]
mod error_text_key;
#[path = "rsa/error_text_padding.rs"]
mod error_text_padding;
#[path = "rsa/key.rs"]
pub mod key;
#[path = "rsa/key_access.rs"]
mod key_access;
#[path = "rsa/key_bytes.rs"]
mod key_bytes;
#[path = "rsa/key_check.rs"]
mod key_check;
#[path = "rsa/pkcs1.rs"]
pub mod pkcs1;
#[path = "rsa/pkcs1_padding.rs"]
mod pkcs1_padding;
#[path = "rsa/verify.rs"]
pub mod verify;

pub use ct::ct_eq;
pub use digestinfo::DigestAlgorithm;
pub use error::RsaError;
pub use key::RsaPublicKey;
pub use pkcs1::check_encoding;
pub use verify::verify;
