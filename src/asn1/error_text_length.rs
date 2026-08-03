//! Messages for length-encoding rule violations.
//!
//! DER admits exactly one encoding per length, so each of these variants marks
//! an input that a permissive BER parser would have accepted.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::error::Error;
//!
//! let err = Error::NonMinimalLength { offset: 1 };
//! assert!(err.to_string().contains("shortest form"));
//! ```

use super::error::Error;

/// Render the length-rule variants, or `None` for any other variant.
///
/// # Arguments
///
/// * `err` — the error to describe.
///
/// # Returns
///
/// `Some(text)` when `err` concerns length encoding, otherwise `None`.
pub(super) fn text(err: &Error) -> Option<String> {
    Some(match err {
        Error::IndefiniteLength { offset } => {
            format!("asn1: indefinite length is forbidden in DER at offset {offset}")
        }
        Error::NonMinimalLength { offset } => format!(
            "asn1: non-minimal length encoding at offset {offset}; DER requires the shortest form"
        ),
        Error::ReservedLength { offset } => {
            format!("asn1: reserved length octet 0xFF at offset {offset}")
        }
        Error::LengthTooLarge { offset, bytes } => format!(
            "asn1: length field of {bytes} byte(s) at offset {offset} is too large to address"
        ),
        _ => return None,
    })
}
