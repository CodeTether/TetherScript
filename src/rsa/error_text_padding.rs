//! # Error text for padding and digest rejections
//!
//! One responsibility: render the [`RsaError`] variants raised while walking the
//! EMSA-PKCS1-v1_5 encoded message. Key-level text lives in
//! `super::error_text_key`.
//!
//! Each message names the octet or length that was wrong, because "invalid
//! padding" alone tells an operator nothing about which relaxation a peer tried.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::rsa::RsaError;
//!
//! let text = format!("{}", RsaError::MissingSeparator);
//! assert!(text.contains("0x00"));
//! ```

use super::error::RsaError;

/// Render a padding-walk or digest-comparison rejection.
///
/// # Arguments
///
/// * `err` — the error to describe.
///
/// # Returns
///
/// `Some(text)` for the variants this stage owns, `None` otherwise.
pub(super) fn padding_text(err: &RsaError) -> Option<String> {
    Some(match err {
        RsaError::EncodingTooShort {
            modulus_bytes,
            needed,
        } => format!(
            "rsa: a {modulus_bytes}-byte modulus cannot hold a {needed}-byte PKCS#1 v1.5 block"
        ),
        RsaError::LeadingBytes { first, second } => format!(
            "rsa: encoded message starts with {first:#04x} {second:#04x}, expected 0x00 0x01"
        ),
        RsaError::PaddingRunTooShort { len } => {
            format!("rsa: 0xff padding run is {len} bytes; at least 8 required")
        }
        RsaError::MissingSeparator => {
            "rsa: 0xff padding run is not terminated by a 0x00 separator".to_string()
        }
        RsaError::DigestInfoLength { expected, found } => format!(
            "rsa: {found} bytes follow the separator but the DigestInfo needs exactly {expected}"
        ),
        RsaError::DigestInfoMismatch => {
            "rsa: DER DigestInfo prefix names a different hash than requested".to_string()
        }
        RsaError::DigestMismatch => "rsa: recovered digest does not match".to_string(),
        RsaError::DigestLength { expected, found } => {
            format!("rsa: digest is {found} bytes but the algorithm produces {expected}")
        }
        _ => return None,
    })
}
