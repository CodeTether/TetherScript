//! Month name lookup and month-length rules.
//!
//! Month length is where the Gregorian leap rule becomes visible to a caller:
//! February has 29 days only when the year is divisible by 4, and not a century
//! unless also divisible by 400. Accepting 1900-02-29 or rejecting 2000-02-29
//! would both be wrong, so the rule is stated once here.

/// Month abbreviations accepted by the IMF-fixdate parser.
const NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Days per month, with February handled separately.
const LENGTHS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Whether `year` is a Gregorian leap year.
///
/// # Arguments
///
/// * `year` — Proleptic Gregorian year.
///
/// # Returns
///
/// True when February has 29 days. Divisible by 4, except centuries that are
/// not divisible by 400: 2000 is a leap year, 1900 is not.
pub(super) fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Last valid day of a month.
///
/// # Arguments
///
/// * `year` — Proleptic Gregorian year, needed for February.
/// * `month` — Month, 1 through 12.
///
/// # Returns
///
/// The final day number, or 0 when `month` is out of range so the caller's
/// range check rejects it.
pub(super) fn valid_day(year: i64, month: i64) -> i64 {
    if !(1..=12).contains(&month) {
        return 0;
    }
    if month == 2 && is_leap_year(year) {
        return 29;
    }
    LENGTHS[(month - 1) as usize]
}

/// Convert a three-letter month abbreviation to its number.
///
/// # Arguments
///
/// * `name` — Abbreviation such as `Oct`, matched case-insensitively.
///
/// # Returns
///
/// The month number, 1 through 12.
///
/// # Errors
///
/// Returns an error naming the unrecognized value.
pub(super) fn month_from_name(name: &str) -> Result<i64, String> {
    NAMES
        .iter()
        .position(|candidate| candidate.eq_ignore_ascii_case(name))
        .map(|index| index as i64 + 1)
        .ok_or_else(|| format!("date parse: `{name}` is not a month abbreviation like `Oct`"))
}
