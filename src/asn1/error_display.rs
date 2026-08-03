//! [`Display`] dispatch for [`Error`].
//!
//! The message text itself lives in three sibling modules grouped by cause so
//! each file keeps a single responsibility: structural framing, length-encoding
//! rules, and tag/content rules.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::error::Error;
//!
//! let err = Error::IndefiniteLength { offset: 1 };
//! assert_eq!(
//!     err.to_string(),
//!     "asn1: indefinite length is forbidden in DER at offset 1"
//! );
//! ```

use std::fmt;

use super::{error::Error, error_text_content, error_text_length, error_text_structure};

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = error_text_structure::text(self)
            .or_else(|| error_text_length::text(self))
            .or_else(|| error_text_content::text(self));
        match text {
            Some(text) => f.write_str(&text),
            None => write!(f, "asn1: decoding failed at offset {}", self.offset()),
        }
    }
}

impl std::error::Error for Error {}
