//! Formatting Unix seconds as IMF-fixdate and RFC 3339.

use super::datetime_civil::{civil_from_days, weekday_from_days};

/// Weekday names, indexed Sunday-first to match `weekday_from_days`.
const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// Month abbreviations, indexed from 1.
const MONTHS: [&str; 13] = [
    "", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Split Unix seconds into a day count and a time of day.
///
/// Uses Euclidean division so pre-epoch timestamps produce a non-negative
/// time of day rather than a negative hour.
///
/// # Arguments
///
/// * `seconds` — Unix timestamp.
///
/// # Returns
///
/// A `(days, hour, minute, second)` tuple.
pub(super) fn split(seconds: i64) -> (i64, i64, i64, i64) {
    let days = seconds.div_euclid(86_400);
    let time_of_day = seconds.rem_euclid(86_400);
    (
        days,
        time_of_day / 3600,
        (time_of_day % 3600) / 60,
        time_of_day % 60,
    )
}

/// Format Unix seconds as an RFC 7231 IMF-fixdate.
///
/// # Arguments
///
/// * `seconds` — Unix timestamp.
///
/// # Returns
///
/// A fixed-width string such as `Wed, 21 Oct 2015 07:28:00 GMT`. HTTP requires
/// exactly this form; the two obsolete formats in RFC 7231 are not emitted.
pub(super) fn http_date(seconds: i64) -> String {
    let (days, hour, minute, second) = split(seconds);
    let (year, month, day) = civil_from_days(days);
    let weekday = DAYS[weekday_from_days(days)];
    let month_name = MONTHS[month as usize];
    format!(
        "{weekday}, {day:02} {month_name} {year:04} \
         {hour:02}:{minute:02}:{second:02} GMT"
    )
}

/// Format Unix seconds as an RFC 3339 timestamp in UTC.
///
/// # Arguments
///
/// * `seconds` — Unix timestamp.
///
/// # Returns
///
/// A string such as `2015-10-21T07:28:00Z`. The zone is always `Z`, since every
/// timestamp in this group is UTC.
pub(super) fn rfc3339(seconds: i64) -> String {
    let (days, hour, minute, second) = split(seconds);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T\
         {hour:02}:{minute:02}:{second:02}Z"
    )
}
