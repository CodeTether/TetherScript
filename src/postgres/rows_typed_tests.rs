//! Type-directed field conversion.
//!
//! Every case here was a real defect before column type OIDs were carried through
//! `RowDescription`: the decoder guessed from the text alone, so a `timestamptz` was an opaque
//! string a script could not compare, and a `varchar` holding digits silently became a number.

use super::binary::oid;
use super::rows_typed::typed;
use crate::value::Value;

/// The defect that motivated the change: a timestamp was unusable as a value.
#[test]
fn a_timestamptz_becomes_unix_seconds() {
    // 2026-03-03T23:00:45Z, the shape the server actually sends: space separator, microseconds,
    // and a two-digit offset.
    let value = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45.517441+00");
    assert_eq!(value, Value::Int(1_772_578_845));
}

/// Fractional seconds truncate rather than round. Rounding up would place an event after
/// something that actually followed it.
#[test]
fn fractional_seconds_truncate() {
    let low = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45.100000+00");
    let high = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45.900000+00");
    assert_eq!(low, high, "both are within the same second");
    assert_eq!(low, Value::Int(1_772_578_845));
}

/// All three offset spellings PostgreSQL emits must resolve to the same instant.
#[test]
fn every_offset_spelling_parses() {
    let bare = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45+00");
    let compact = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45+0000");
    let colon = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45+00:00");
    assert_eq!(bare, compact);
    assert_eq!(bare, colon);
}

/// A non-UTC offset must shift the instant, not be ignored.
#[test]
fn a_non_utc_offset_shifts_the_instant() {
    let utc = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45+00");
    let minus_five = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45-05");
    let Value::Int(utc) = utc else {
        panic!("not an int")
    };
    let Value::Int(shifted) = minus_five else {
        panic!("not an int")
    };
    // 23:00:45 at UTC-5 is five hours later in absolute terms.
    assert_eq!(shifted - utc, 5 * 3600);
}

#[test]
fn an_rfc3339_style_t_separator_also_parses() {
    let spaced = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45+00");
    let tee = typed(oid::TIMESTAMPTZ, "2026-03-03T23:00:45+00");
    assert_eq!(spaced, tee);
}

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

/// The leap-year rule that divides by 400, which a naive implementation gets wrong.
#[test]
fn leap_days_are_exact() {
    // 2000 was a leap year; 1900 was not.
    assert_eq!(typed(oid::DATE, "2000-02-29"), Value::Int(951_782_400));
    assert_eq!(typed(oid::DATE, "2024-02-29"), Value::Int(1_709_164_800));
}

/// The second defect: a declared-textual column must keep its text, whatever it contains. A
/// product code or zero-padded identifier silently became a number before.
#[test]
fn a_numeric_looking_varchar_stays_a_string() {
    let value = typed(oid::VARCHAR, "0123");
    assert_eq!(value, Value::Str("0123".to_string().into()));
}

/// `numeric` is arbitrary precision, so it stays text: parsing to f64 would round a money column.
#[test]
fn numeric_stays_text_so_money_is_not_rounded() {
    let value = typed(oid::NUMERIC, "12345.678901234567890");
    assert_eq!(
        value,
        Value::Str("12345.678901234567890".to_string().into())
    );
}

#[test]
fn the_ordinary_scalars_still_convert() {
    assert_eq!(typed(oid::INT4, "42"), Value::Int(42));
    assert_eq!(typed(oid::INT8, "-9000"), Value::Int(-9000));
    assert_eq!(typed(oid::FLOAT8, "1.5"), Value::Float(1.5));
    assert_eq!(typed(oid::BOOL, "t"), Value::Bool(true));
    assert_eq!(typed(oid::BOOL, "f"), Value::Bool(false));
}

/// An unparsable value in a typed column is returned as text rather than as a wrong number: a
/// silently shifted date is far worse than one a script can see is unconverted.
#[test]
fn an_unparsable_value_falls_back_to_text() {
    assert_eq!(
        typed(oid::TIMESTAMPTZ, "infinity"),
        Value::Str("infinity".to_string().into())
    );
    assert_eq!(
        typed(oid::INT4, "not-a-number"),
        Value::Str("not-a-number".to_string().into())
    );
}

/// An unrecognised OID still infers, so a type this table does not know is usable rather than
/// opaque. That is the behaviour every column had before.
#[test]
fn an_unknown_oid_still_infers() {
    assert_eq!(typed(0, "42"), Value::Int(42));
    assert_eq!(typed(0, "t"), Value::Bool(true));
    assert_eq!(typed(0, "hello"), Value::Str("hello".to_string().into()));
}
