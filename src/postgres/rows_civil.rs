//! Civil-date to epoch-day conversion.
//!
//! Howard Hinnant's `days_from_civil`, exact across the whole `i64` range and needing no lookup
//! tables. The binary decoder has its own copy against a different epoch; this one is against the
//! Unix epoch, and duplicating thirty lines is preferable to widening a `pub(super)` boundary
//! between two modules that only coincidentally agree today.

/// Days from 1970-01-01 to the given civil date, negative for earlier dates.
///
/// # Arguments
///
/// * `year`, `month` (1-12), `day` (1-31).
///
/// # Returns
///
/// Signed day count. Out-of-range month or day values still produce a number rather than
/// panicking, because the input came off a network socket.
pub(super) fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    // Shift the year to start in March, which moves the leap day to the end of the cycle and
    // removes every February special case from the arithmetic below.
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let shifted_month = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}
