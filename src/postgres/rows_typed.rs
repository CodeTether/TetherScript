//! Type-directed conversion of a text-format field to a script value.
//!
//! The OID decides the conversion, so a column's declared type governs rather than the shape its
//! current value happens to have.

use std::rc::Rc;

use super::binary::oid;
use crate::value::Value;

/// Convert one text-format field using its column's type OID.
///
/// # Arguments
///
/// * `type_oid` — PostgreSQL type OID from `RowDescription`, or 0 when unknown.
/// * `text` — The field's text-format bytes as UTF-8.
///
/// # Returns
///
/// An `Int` for integer columns, `Float` for reals, `Bool` for booleans, `Int` of Unix seconds for
/// temporals, and `Str` for everything else — including a numeric-looking `varchar`, which must
/// stay a string or a leading zero is lost.
pub(super) fn typed(type_oid: u32, text: &str) -> Value {
    match type_oid {
        oid::BOOL => boolean(text),
        oid::INT2 | oid::INT4 | oid::INT8 => integer(text),
        oid::FLOAT4 | oid::FLOAT8 => real(text),
        // `numeric` is arbitrary precision, so it stays text: parsing it as f64 would silently
        // round a money column, which is the one place that is never acceptable.
        oid::NUMERIC => Value::Str(Rc::new(text.to_string())),
        oid::TIMESTAMP | oid::TIMESTAMPTZ | oid::DATE => temporal(type_oid, text),
        // Declared textual: return it verbatim, whatever it looks like.
        oid::TEXT | oid::VARCHAR | oid::BPCHAR | oid::NAME | oid::JSON | oid::JSONB | oid::UUID => {
            Value::Str(Rc::new(text.to_string()))
        }
        // Unknown OID, including 0: fall back to inference so an unrecognised type is still
        // usable rather than opaque.
        _ => super::rows_infer::infer(text),
    }
}

/// PostgreSQL renders booleans as `t` and `f` in text format.
fn boolean(text: &str) -> Value {
    match text {
        "t" => Value::Bool(true),
        "f" => Value::Bool(false),
        other => Value::Str(Rc::new(other.to_string())),
    }
}

/// An integer column that will not parse is returned as text rather than silently zeroed.
fn integer(text: &str) -> Value {
    match text.parse::<i64>() {
        Ok(number) => Value::Int(number),
        Err(_) => Value::Str(Rc::new(text.to_string())),
    }
}

/// A float column that will not parse is returned as text.
fn real(text: &str) -> Value {
    match text.parse::<f64>() {
        Ok(number) => Value::Float(number),
        Err(_) => Value::Str(Rc::new(text.to_string())),
    }
}

/// Convert a temporal column to Unix seconds.
///
/// Unix seconds because that is what every date built-in in the language takes: `rfc3339`,
/// `http_date`, and arithmetic against `time_now_secs()` all speak it. Returning the server's
/// text would leave a script unable to compare two timestamps without writing a parser.
///
/// A value that cannot be parsed is returned as text rather than as a wrong number, since a
/// silently shifted date is far worse than one a script can see is unconverted.
fn temporal(type_oid: u32, text: &str) -> Value {
    match super::rows_time::unix_seconds(type_oid, text) {
        Some(seconds) => Value::Int(seconds),
        None => Value::Str(Rc::new(text.to_string())),
    }
}
