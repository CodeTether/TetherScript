//! Proleptic Gregorian calendar conversion between Unix days and civil dates.
//!
//! This is the load-bearing arithmetic for every date built-in in this group. It
//! is Howard Hinnant's `days_from_civil` / `civil_from_days` algorithm, which is
//! exact for the full proleptic Gregorian calendar and needs no lookup tables.
//!
//! The full Gregorian leap rule is inherent to the algorithm rather than bolted
//! on: a year is a leap year when it is divisible by 4, **except** centuries not
//! divisible by 400. So 2000 is a leap year and 1900 is not. An off-by-one here
//! would silently write cookie expiries into the wrong year, which is why the
//! conversion lives in its own module with its own tests.
//!
//! Both directions shift the era to start on 1 March, which moves the leap day
//! to the end of the year and removes February as a special case.

/// Days from 1970-01-01 to the given civil date.
///
/// # Arguments
///
/// * `year` — Proleptic Gregorian year, may be negative.
/// * `month` — Month, 1 through 12.
/// * `day` — Day of month, 1 through 31.
///
/// # Returns
///
/// Signed day count relative to the Unix epoch; negative before 1970.
pub(super) fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    // Treat March as the first month so the leap day lands last.
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_shift = if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * (month + month_shift) + 2) / 5 + day - 1;
    // Leap days within the era: every 4th year, minus centuries, plus 400s.
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Civil date for a day count relative to 1970-01-01.
///
/// # Arguments
///
/// * `days` — Signed day count; negative before 1970.
///
/// # Returns
///
/// A `(year, month, day)` triple, with month 1-12 and day 1-31.
pub(super) fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Day of the week for a Unix day count.
///
/// Computed, never guessed: 1970-01-01 was a Thursday, so index 4 in a
/// Sunday-first week. Euclidean remainder keeps pre-epoch days non-negative.
///
/// # Arguments
///
/// * `days` — Signed day count relative to 1970-01-01.
///
/// # Returns
///
/// Weekday index, 0 = Sunday through 6 = Saturday.
pub(super) fn weekday_from_days(days: i64) -> usize {
    (days + 4).rem_euclid(7) as usize
}
