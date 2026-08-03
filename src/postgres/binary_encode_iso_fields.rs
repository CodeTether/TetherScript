//! # Field splitting and range validation for ISO-8601 input
//!
//! The lexical half of ISO parsing, kept apart from the epoch arithmetic so the
//! validation rules are reviewable on their own. Every field is range-checked, since
//! `days_from_civil` accepts whatever it is given and would happily turn month 13
//! into a date in the following year.
//!
//! Splitting uses `split` and `parse`, never byte indexing, so a short or
//! multi-byte-character input cannot panic — this is still parsing text that came from
//! a script and, sometimes, ultimately from a user.
//!
//! The `time` half and the shared error/number helpers are siblings:
//! `binary_encode_iso_time.rs` and `binary_encode_iso_util.rs`.

use super::super::super::super::error::DecodeError;
use super::util::{bad, number};

/// Split `YYYY-MM-DD` into validated `(year, month, day)`.
///
/// # Arguments
///
/// * `text` — the trimmed date text.
///
/// # Returns
///
/// `(year, month, day)` with month in 1..=12 and day in 1..=31.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for the wrong field count, a non-numeric field, or a
/// field out of range. A BC year written with a leading `-` is rejected rather than
/// misread, since it would split into four parts.
pub(super) fn split_date(text: &str) -> Result<(i64, u32, u32), DecodeError> {
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 3 {
        return Err(bad("date", text, "expected YYYY-MM-DD"));
    }
    let year: i64 = number(parts[0], "date", text)?;
    let month: u32 = number(parts[1], "date", text)?;
    let day: u32 = number(parts[2], "date", text)?;
    if !(1..=12).contains(&month) {
        return Err(bad("date", text, "month must be 01..=12"));
    }
    if !(1..=31).contains(&day) {
        return Err(bad("date", text, "day must be 01..=31"));
    }
    Ok((year, month, day))
}

/// Split a timestamp on its single `T` or space separator.
///
/// # Arguments
///
/// * `text` — the trimmed timestamp text, with any `Z` already removed.
///
/// # Returns
///
/// `(date_part, time_part)`.
///
/// # Errors
///
/// [`DecodeError::BadValue`] when no separator is present.
pub(super) fn split_datetime(text: &str) -> Result<(&str, &str), DecodeError> {
    text.split_once('T')
        .or_else(|| text.split_once(' '))
        .ok_or_else(|| bad("timestamp", text, "expected a 'T' or space before the time"))
}
