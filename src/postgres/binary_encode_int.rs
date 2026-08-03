//! # Integer parameter encoders
//!
//! Each writes exactly its type's width in **big-endian** two's complement, via
//! `to_be_bytes`. Nothing here is a cast that could silently wrap: tetherscript's
//! `Int` is `i64`, and `int2`/`int4`/`oid` are narrower, so a value out of range is
//! **rejected by name** rather than truncated. Truncating `70000` into an `int2` as
//! `4464` would be a data-corruption bug that no test downstream could catch.
//!
//! `oid` is unsigned, so its accepted range is `0..=4_294_967_295` and it is written
//! through `u32::to_be_bytes`.

use crate::value::Value;

use super::super::error::DecodeError;
use super::mismatch;

/// Encode an `int2`: 2 big-endian bytes.
///
/// # Arguments
///
/// * `value` — a [`Value::Int`] within `i16` range.
///
/// # Returns
///
/// 2 bytes, big-endian.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-int value or one outside `i16` range.
pub(super) fn int2(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let int = require_int(value, "int2")?;
    let narrow = i16::try_from(int).map_err(|_| out_of_range("int2", int))?;
    Ok(narrow.to_be_bytes().to_vec())
}

/// Encode an `int4`: 4 big-endian bytes.
///
/// # Arguments
///
/// * `value` — a [`Value::Int`] within `i32` range.
///
/// # Returns
///
/// 4 bytes, big-endian.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-int value or one outside `i32` range.
pub(super) fn int4(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let int = require_int(value, "int4")?;
    let narrow = i32::try_from(int).map_err(|_| out_of_range("int4", int))?;
    Ok(narrow.to_be_bytes().to_vec())
}

/// Encode an `int8`: 8 big-endian bytes. Every `Int` fits, so this cannot overflow.
///
/// # Arguments
///
/// * `value` — a [`Value::Int`].
///
/// # Returns
///
/// 8 bytes, big-endian.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-int value.
pub(super) fn int8(value: &Value) -> Result<Vec<u8>, DecodeError> {
    Ok(require_int(value, "int8")?.to_be_bytes().to_vec())
}

/// Encode an `oid`: 4 big-endian **unsigned** bytes.
///
/// # Arguments
///
/// * `value` — a [`Value::Int`] in `0..=4_294_967_295`.
///
/// # Returns
///
/// 4 bytes, big-endian.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-int value or one outside `u32` range,
/// including any negative value.
pub(super) fn oid_value(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let int = require_int(value, "oid")?;
    let narrow = u32::try_from(int).map_err(|_| out_of_range("oid", int))?;
    Ok(narrow.to_be_bytes().to_vec())
}

/// Extract an `i64` or report the type mismatch by name.
fn require_int(value: &Value, what: &'static str) -> Result<i64, DecodeError> {
    match value {
        Value::Int(int) => Ok(*int),
        // Bool is deliberately not coerced: binding `true` to an integer column is
        // almost always a mistake, and silently sending 1 would hide it.
        other => Err(mismatch(what, other)),
    }
}

/// Report an out-of-range integer, naming the type and the offending value.
fn out_of_range(what: &'static str, int: i64) -> DecodeError {
    DecodeError::BadValue {
        what,
        detail: format!("{int} is outside the range of {what}; it would wrap if truncated"),
    }
}
