//! # ISO-8601 rendering of binary date/time counters
//!
//! A binary `timestamp` is an integer counter, but a script wants something it can
//! print, compare, and hand straight back to PostgreSQL. So the decoders render
//! ISO-8601 text: `2024-01-15T10:30:00Z` for `timestamptz`, the same without `Z`
//! for `timestamp`, `2024-01-15` for `date`, `10:30:00.123456` for `time`.
//!
//! Two invariants make this safe:
//!
//! - **Euclidean split.** Splitting microseconds into a day number and a
//!   time-of-day uses `div_euclid`/`rem_euclid`, not `/` and `%`. Rust's `%`
//!   truncates toward zero, so a pre-1970 instant would yield a *negative*
//!   time-of-day and render nonsense like `1969-12-31T-1:00:00`. Euclidean
//!   division always leaves a non-negative remainder.
//! - **Fractional seconds are dropped only when zero.** `10:30:00` and
//!   `10:30:00.000001` are different instants, so the microsecond field is emitted
//!   whenever it is non-zero, with trailing zeros trimmed.
//!
//! `infinity` and `-infinity` arrive as `i64::MAX`/`i64::MIN` (and the `i32`
//! extremes for `date`) and are rendered by those names rather than as an absurd
//! year far outside the calendar.

use super::super::super::time::civil::civil_from_days;
use super::super::super::time::{date_unix_days, timestamp_unix_micros};

/// Microseconds in one 24-hour day. `time` and `timestamp` both split on this.
const MICROS_PER_DAY: i64 = 86_400 * 1_000_000;

/// Render a PostgreSQL timestamp counter as ISO-8601.
///
/// # Arguments
///
/// * `pg_micros` — microseconds since 2000-01-01, read big-endian off the wire.
/// * `utc` — append `Z`; true for `timestamptz`.
///
/// # Returns
///
/// `YYYY-MM-DDTHH:MM:SS[.ffffff][Z]`, or `infinity`/`-infinity`.
pub(super) fn timestamp(pg_micros: i64, utc: bool) -> String {
    if pg_micros == i64::MAX {
        return "infinity".into();
    }
    if pg_micros == i64::MIN {
        return "-infinity".into();
    }
    let unix = timestamp_unix_micros(pg_micros);
    // Euclidean so a pre-1970 instant keeps a non-negative time-of-day.
    let days = unix.div_euclid(MICROS_PER_DAY);
    let micros = unix.rem_euclid(MICROS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    let suffix = if utc { "Z" } else { "" };
    let clock = clock(micros as u64);
    format!("{year:04}-{month:02}-{day:02}T{clock}{suffix}")
}

/// Render a PostgreSQL `date` counter as `YYYY-MM-DD`.
///
/// # Arguments
///
/// * `pg_days` — days since 2000-01-01.
///
/// # Returns
///
/// `YYYY-MM-DD`, or `infinity`/`-infinity` for the sentinel extremes.
pub(super) fn date(pg_days: i32) -> String {
    if pg_days == i32::MAX {
        return "infinity".into();
    }
    if pg_days == i32::MIN {
        return "-infinity".into();
    }
    let (year, month, day) = civil_from_days(date_unix_days(pg_days) as i64);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Render a `time` value: microseconds since midnight as `HH:MM:SS[.ffffff]`.
///
/// # Arguments
///
/// * `micros` — microseconds since midnight, folded into a single day so a
///   malformed value cannot produce an hour field in the thousands.
///
/// # Returns
///
/// `HH:MM:SS`, with a fractional part when the microseconds are non-zero.
pub(super) fn time_of_day(micros: i64) -> String {
    clock(micros.rem_euclid(MICROS_PER_DAY) as u64)
}

/// `HH:MM:SS` plus a trimmed fractional part when the microseconds are non-zero.
fn clock(micros: u64) -> String {
    let seconds = micros / 1_000_000;
    let fraction = micros % 1_000_000;
    let hours = seconds / 3_600;
    let minutes = (seconds / 60) % 60;
    let base = format!("{hours:02}:{minutes:02}:{:02}", seconds % 60);
    if fraction == 0 {
        return base;
    }
    let digits = format!("{fraction:06}");
    format!("{base}.{}", digits.trim_end_matches('0'))
}
