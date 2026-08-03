//! Temporal conversion: the defect that motivated carrying type OIDs through.
//!
//! Before the OID was available, a `timestamptz` was an opaque string a script could not compare
//! or format. These lock in that it becomes Unix seconds, which is what every date built-in takes.

use super::binary::oid;
use super::rows_typed::typed;
use crate::value::Value;

#[test]
fn a_timestamptz_becomes_unix_seconds() {
    // The shape the server actually sends: space separator, microseconds, two-digit offset.
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
    let Value::Int(utc) = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45+00") else {
        panic!("not an int")
    };
    let Value::Int(shifted) = typed(oid::TIMESTAMPTZ, "2026-03-03 23:00:45-05") else {
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
