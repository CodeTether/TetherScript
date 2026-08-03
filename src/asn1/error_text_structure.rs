//! Messages for structural framing failures (truncation, overrun, PEM armour).
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::error::Error;
//!
//! let err = Error::LengthExceedsInput {
//!     offset: 2,
//!     length: 64,
//!     available: 3,
//! };
//! assert!(err.to_string().contains("exceeds the 3 byte(s) remaining"));
//! ```

use super::error::Error;

/// Render the structural variants, or `None` for any other variant.
///
/// # Arguments
///
/// * `err` — the error to describe.
///
/// # Returns
///
/// `Some(text)` when `err` is a structural variant, otherwise `None`.
pub(super) fn text(err: &Error) -> Option<String> {
    Some(match err {
        Error::UnexpectedEnd { offset } => {
            format!("asn1: input ended unexpectedly at offset {offset}")
        }
        Error::LengthExceedsInput {
            offset,
            length,
            available,
        } => format!(
            "asn1: length {length} at offset {offset} exceeds the {available} byte(s) remaining"
        ),
        Error::TrailingData { offset } => {
            format!("asn1: unexpected trailing data at offset {offset}")
        }
        Error::Pem { offset, reason } => format!("asn1: pem error at offset {offset}: {reason}"),
        _ => return None,
    })
}
