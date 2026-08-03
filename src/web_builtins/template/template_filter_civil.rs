//! Civil-date conversion for the `date` filter.
//!
//! Howard Hinnant's `civil_from_days` algorithm, which is exact for the whole range of
//! `i64` days and needs no lookup tables. Duplicated from the `datetime` group rather than
//! sharing it, because those helpers are `pub(super)` to a different parent and widening
//! their visibility for one caller would be the worse trade.

/// Convert days since the Unix epoch to a `(year, month, day)` civil date.
///
/// # Arguments
///
/// * `days` — Days since 1970-01-01, negative for earlier dates.
///
/// # Returns
///
/// Year, month (1-12), and day (1-31).
pub(super) fn civil_from_days(days: i64) -> (i64, i64, i64) {
    // Shift the epoch to 0000-03-01, which puts the leap day at the end of the cycle and
    // removes every February special case from the arithmetic.
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
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
