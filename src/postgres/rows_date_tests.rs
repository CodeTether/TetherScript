//! `date` conversion, including the calendar edge cases.
//!
//! Split from `rows_temporal_tests` so each file stays within the line budget.

use super::binary::oid;
use super::rows_typed::typed;
use crate::value::Value;

/// A `date` is midnight UTC on that day.
#[test]
fn a_date_becomes_midnight_unix_seconds() {
    assert_eq!(typed(oid::DATE, "2026-03-03"), Value::Int(1_772_496_000));
    assert_eq!(typed(oid::DATE, "1970-01-01"), Value::Int(0));
}

/// Dates before the epoch are negative, not wrapped.
#[test]
fn a_pre_epoch_date_is_negative() {
    assert_eq!(typed(oid::DATE, "1969-12-31"), Value::Int(-86_400));
}

/// The leap-year rule that divides by 400, which a naive implementation gets wrong: 2000 was a leap
/// year and 1900 was not, so an implementation checking only divisibility by 4 drifts by a day.
#[test]
fn leap_days_are_exact() {
    assert_eq!(typed(oid::DATE, "2000-02-29"), Value::Int(951_782_400));
    assert_eq!(typed(oid::DATE, "2024-02-29"), Value::Int(1_709_164_800));
}
