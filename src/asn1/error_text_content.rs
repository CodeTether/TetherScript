//! Messages for tag, content, and nesting-limit violations.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::error::Error;
//!
//! let err = Error::UnexpectedTag {
//!     offset: 0,
//!     expected: 0x30,
//!     found: 0x02,
//! };
//! assert!(err.to_string().contains("found 0x02"));
//! ```

use super::error::Error;

/// Render the tag/content/depth variants, or `None` for any other variant.
///
/// # Arguments
///
/// * `err` — the error to describe.
///
/// # Returns
///
/// `Some(text)` when `err` concerns a tag, its content, or nesting depth.
pub(super) fn text(err: &Error) -> Option<String> {
    Some(match err {
        Error::HighTagNumber { offset } => {
            format!("asn1: high-tag-number form at offset {offset} is not supported")
        }
        Error::UnexpectedTag {
            offset,
            expected,
            found,
        } => format!(
            "asn1: expected tag 0x{expected:02x} at offset {offset}, found 0x{found:02x}"
        ),
        Error::DepthExceeded { offset, max_depth } => {
            format!("asn1: nesting deeper than {max_depth} at offset {offset}")
        }
        Error::MalformedValue {
            offset,
            tag,
            reason,
        } => format!("asn1: malformed value with tag 0x{tag:02x} at offset {offset}: {reason}"),
        _ => return None,
    })
}
