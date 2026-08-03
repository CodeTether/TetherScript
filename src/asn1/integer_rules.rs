//! The INTEGER content-octet rules, separated from the reader method so the
//! rule set can be tested and reasoned about on its own.
//!
//! Splitting the rules out keeps [`super::integer`] responsible only for
//! plumbing a TLV into a value, while this file owns the encoding policy.

use super::{error::Error, tag};

/// Reject content octets that are not a minimal DER INTEGER encoding.
///
/// # Arguments
///
/// * `content` — the INTEGER's content octets.
/// * `offset` — absolute offset of the INTEGER's tag, for error reporting.
///
/// # Errors
///
/// [`Error::MalformedValue`] for empty content, an illegal leading `0x00`, or an
/// illegal leading `0xFF`.
///
/// # Panics
///
/// Never; the octets are matched by slice pattern rather than indexed.
pub(super) fn check(content: &[u8], offset: usize) -> Result<(), Error> {
    let bad = |reason| Error::MalformedValue {
        offset,
        tag: tag::INTEGER,
        reason,
    };
    match content {
        [] => Err(bad("INTEGER must have at least one content octet")),
        [0x00, second, ..] if *second < 0x80 => {
            Err(bad("INTEGER has an illegal leading zero octet"))
        }
        [0xff, second, ..] if *second >= 0x80 => {
            Err(bad("INTEGER has an illegal leading 0xFF octet"))
        }
        _ => Ok(()),
    }
}

/// Convert already-validated content octets to a `u64`.
///
/// # Arguments
///
/// * `content` — minimal INTEGER content octets.
/// * `offset` — absolute offset of the INTEGER's tag.
///
/// # Returns
///
/// The non-negative value the octets denote.
///
/// # Errors
///
/// [`Error::MalformedValue`] when the value is negative or too wide for a `u64`.
///
/// # Panics
///
/// Never; the fold shifts at most eight times after the width check.
pub(super) fn to_u64(content: &[u8], offset: usize) -> Result<u64, Error> {
    let bad = |reason| Error::MalformedValue {
        offset,
        tag: tag::INTEGER,
        reason,
    };
    let digits = match content {
        [first, ..] if *first >= 0x80 => {
            return Err(bad("INTEGER is negative where unsigned was required"))
        }
        [0x00, rest @ ..] => rest,
        other => other,
    };
    if digits.len() > 8 {
        return Err(bad("INTEGER is too large for a 64-bit unsigned value"));
    }
    Ok(digits
        .iter()
        .fold(0u64, |acc, byte| (acc << 8) | u64::from(*byte)))
}
