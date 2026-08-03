//! # Parsing a padded digit string into base-10000 groups
//!
//! The final, purely mechanical step of `numeric` encoding: take a digit string whose
//! length is already a multiple of four and turn it into `u16` groups.
//!
//! Because each chunk is exactly four ASCII digits, the maximum possible value is 9999
//! and the parse cannot overflow `u16`. The error arm exists only so this file contains
//! no `unwrap` — the same no-panic discipline the decoders follow, applied here even
//! though the input is locally produced.

use super::super::super::error::DecodeError;
use super::groups::bad;

/// Parse a digit string whose length is a multiple of 4 into `u16` groups.
///
/// # Arguments
///
/// * `padded` — ASCII digits, length already a multiple of 4.
///
/// # Returns
///
/// One group per four digits, most significant first.
///
/// # Errors
///
/// [`DecodeError::BadValue`] if a chunk is not four ASCII digits, which the caller's
/// validation should already have ruled out.
pub(super) fn parse_groups(padded: &str) -> Result<Vec<u16>, DecodeError> {
    padded
        .as_bytes()
        .chunks(4)
        .map(|chunk| {
            let text = std::str::from_utf8(chunk).unwrap_or("");
            text.parse::<u16>()
                .map_err(|_| bad(format!("{text:?} is not a 4-digit group")))
        })
        .collect()
}
