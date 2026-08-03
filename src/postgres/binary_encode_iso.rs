//! # Parsing ISO-8601 date/time strings for parameter binding
//!
//! The exact inverse of `binary_decode_iso.rs`, so a timestamp read from one query
//! binds unchanged into the next. That round-trip property is the reason both
//! directions are in-tree rather than delegated: there is no dependency to keep them
//! consistent, only a test.
//!
//! Accepted shapes, deliberately narrow — a parser that guesses is a parser that
//! silently accepts the wrong date:
//!
//! ```text
//! date       YYYY-MM-DD
//! time       HH:MM[:SS[.ffffff]]
//! timestamp  YYYY-MM-DD['T'|' ']HH:MM[:SS[.ffffff]]['Z']
//! ```
//!
//! Fractional seconds are right-padded to exactly six digits, so `.5` means
//! 500 000 µs, not 5. Truncating instead would turn half a second into five
//! microseconds. More than six digits is rejected rather than rounded, because
//! `numeric`-style silent precision loss in a timestamp is the same class of bug.
//!
//! Field ranges are validated: month 1..=12, day 1..=31, hour 0..=23, minute and
//! second 0..=59. A leap second (`:60`) is rejected, matching PostgreSQL.

use super::super::super::error::DecodeError;
use super::super::super::time::civil::days_from_civil;

#[path = "binary_encode_iso_fields.rs"]
mod fields;
#[path = "binary_encode_iso_time.rs"]
mod time;
#[path = "binary_encode_iso_util.rs"]
mod util;

/// Parse `YYYY-MM-DD` into days since the **Unix** epoch.
///
/// # Arguments
///
/// * `text` — an ISO-8601 calendar date.
///
/// # Returns
///
/// Days since 1970-01-01. The caller applies the PostgreSQL epoch shift.
///
/// # Errors
///
/// [`DecodeError::BadValue`] naming the offending text when the shape or any field
/// range is wrong.
pub(super) fn parse_date(text: &str) -> Result<i64, DecodeError> {
    let (year, month, day) = fields::split_date(text.trim())?;
    Ok(days_from_civil(year, month, day))
}

/// Parse `HH:MM[:SS[.ffffff]]` into microseconds since midnight.
///
/// # Arguments
///
/// * `text` — an ISO-8601 wall-clock time.
///
/// # Returns
///
/// Microseconds since midnight, in `0..86_400_000_000`.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a malformed or out-of-range time.
pub(super) fn parse_time(text: &str) -> Result<i64, DecodeError> {
    time::split_time(text.trim())
}

/// Parse a full timestamp into microseconds since the **Unix** epoch.
///
/// # Arguments
///
/// * `text` — `YYYY-MM-DD` then `T` or a space, then the time, optionally `Z`.
///
/// # Returns
///
/// Microseconds since 1970-01-01. The caller applies the PostgreSQL epoch shift.
///
/// # Errors
///
/// [`DecodeError::BadValue`] when either half is malformed, or when the two are not
/// separated by exactly one `T` or space.
pub(super) fn parse_timestamp(text: &str) -> Result<i64, DecodeError> {
    let trimmed = text.trim().trim_end_matches('Z');
    let (date_part, time_part) = fields::split_datetime(trimmed)?;
    let days = parse_date(date_part)?;
    let micros = parse_time(time_part)?;
    // 86_400_000_000 µs per day; an i64 spans ~292 000 years, so this is safe.
    Ok(days.saturating_mul(86_400_000_000).saturating_add(micros))
}
