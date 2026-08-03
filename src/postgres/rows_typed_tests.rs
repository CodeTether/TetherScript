//! Type-directed field conversion for non-temporal columns.
//!
//! Temporal cases live in `rows_temporal_tests`. Every case here was a real defect before column
//! type OIDs were carried through `RowDescription`: the decoder guessed from the text alone, so a
//! `varchar` holding digits silently became a number.

use super::binary::oid;
use super::rows_typed::typed;
use crate::value::Value;

/// A declared-textual column must keep its text, whatever it contains. A product code or
/// zero-padded identifier silently became a number before.
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
