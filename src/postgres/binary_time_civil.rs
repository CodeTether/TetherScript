//! # Proleptic Gregorian calendar arithmetic
//!
//! Rendering a `date` or `timestamp` as an ISO-8601 string needs a day-number to
//! year/month/day conversion, and tetherscript has **zero required
//! dependencies** — no `chrono`, no `time`. So the conversion is in-tree, using
//! Howard Hinnant's `civil_from_days` / `days_from_civil` pair (public domain,
//! `howardhinnant.github.io/date_algorithms.html`).
//!
//! The algorithm is exact across the whole proleptic Gregorian calendar,
//! including the 100/400-year leap rules, and uses only integer arithmetic — no
//! floating point, so no rounding drift. Both directions are implemented because
//! the same table is needed to *encode* a date parameter, and a round-trip test
//! over a wide day range is the cheapest proof the two tables agree.
//!
//! Day zero is 1970-01-01 (the Unix epoch), so callers convert the PostgreSQL
//! 2000-based counter with
//! [`date_unix_days`](crate::postgres::binary::date_unix_days) first.
//!
//! Every intermediate is `i64` and every division is Rust's truncating integer
//! division. The `biased` bindings exist so the `if`/`else` result is divided as a
//! whole, which is both correct and unambiguous to read.

/// Split a Unix day number into a `(year, month, day)` civil date.
///
/// # Arguments
///
/// * `days` — days since 1970-01-01; negative values reach back before it.
///
/// # Returns
///
/// `(year, month, day)` with `month` in 1..=12 and `day` in 1..=31.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::civil_from_days;
///
/// assert_eq!(civil_from_days(0), (1970, 1, 1));
/// assert_eq!(civil_from_days(10_957), (2000, 1, 1));
/// assert_eq!(civil_from_days(19_737), (2024, 1, 15));
/// ```
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    // Shift to an era starting 0000-03-01 so the leap day lands at the era end.
    let z = days + 719_468;
    let biased = if z >= 0 { z } else { z - 146_096 };
    let era = biased / 146_097;
    let doe = z - era * 146_097; // day of era, 0..=146_096
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // 0..=399
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year, 0..=365
    let mp = (5 * doy + 2) / 153; // March-based month index, 0..=11
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let march_year = yoe + era * 400;
    let year = if month <= 2 { march_year + 1 } else { march_year };
    (year, month, day)
}

/// Fold a `(year, month, day)` civil date into a Unix day number.
///
/// # Arguments
///
/// * `year` — proleptic Gregorian year.
/// * `month` — 1..=12. Out-of-range input yields an unspecified but finite day
///   number rather than a panic; callers validate before calling.
/// * `day` — 1..=31, likewise unvalidated here.
///
/// # Returns
///
/// Days since 1970-01-01. Exact inverse of [`civil_from_days`] for every valid
/// civil date.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{civil_from_days, days_from_civil};
///
/// assert_eq!(days_from_civil(1970, 1, 1), 0);
/// assert_eq!(days_from_civil(2024, 1, 15), 19_737);
/// // The leap-year boundary round-trips.
/// assert_eq!(civil_from_days(days_from_civil(2000, 2, 29)), (2000, 2, 29));
/// ```
pub fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let month = month as i64;
    // January and February belong to the previous March-based year.
    let shifted = if month <= 2 { year - 1 } else { year };
    let biased = if shifted >= 0 { shifted } else { shifted - 399 };
    let era = biased / 400;
    let yoe = shifted - era * 400; // 0..=399
    let march_month = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * march_month + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}
