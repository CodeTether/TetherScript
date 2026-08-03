//! Parsing PostgreSQL text-format timestamps into Unix seconds.
//!
//! Text format for `timestamptz` is `2026-03-03 23:00:45.517441+00`: a space separator rather than
//! RFC 3339's `T`, optional fractional seconds, and an offset that may be `+00`, `+0000`, or
//! `+00:00`. `date` is a bare `2026-03-03`.
//!
//! Fractional seconds are truncated, not rounded: a timestamp is being reduced to second
//! granularity, and rounding up would place an event after something that actually followed it.

use super::binary::oid;

/// Convert a text-format temporal value to Unix seconds.
///
/// # Arguments
///
/// * `type_oid` — `DATE`, `TIMESTAMP`, or `TIMESTAMPTZ`.
/// * `text` — The server's text rendering.
///
/// # Returns
///
/// `None` when the text does not parse, so the caller can hand back the original rather than a
/// wrong number.
pub(super) fn unix_seconds(type_oid: u32, text: &str) -> Option<i64> {
    let text = text.trim();
    if type_oid == oid::DATE {
        let (year, month, day) = super::rows_time_parts::date_parts(text)?;
        return Some(super::rows_civil::days_from_civil(year, month, day) * 86_400);
    }
    let (date, rest) = text.split_once([' ', 'T'])?;
    let (year, month, day) = super::rows_time_parts::date_parts(date)?;
    let days = super::rows_civil::days_from_civil(year, month, day);

    let (clock, offset) = super::rows_time_parts::split_offset(rest);
    let (hour, minute, second) = super::rows_time_parts::clock_parts(clock)?;
    // `timestamp` without a zone is taken as UTC, matching how the binary decoder treats it: the
    // server stores no zone, so there is nothing else to assume.
    Some(days * 86_400 + hour * 3600 + minute * 60 + second - offset)
}
