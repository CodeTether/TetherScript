//! # `bool`, `float4`, and `float8` parameter encoders
//!
//! `bool` is a single byte, `0` or `1`. The floats are written as raw IEEE-754 bit
//! patterns, **big-endian**, via `to_bits().to_be_bytes()`.
//!
//! An `Int` is accepted for a float column and widened, because binding `1` to a
//! `float8` is unambiguous and common. The reverse is *not* allowed: an `Int`
//! parameter never becomes a float and a `Float` never becomes an integer, since
//! that would silently drop a fractional part.
//!
//! `float4` narrows `f64` to `f32`, which is lossy by definition of the target
//! type — that is what the caller asked for by naming `float4` — but the cast is
//! called out here so it is a decision rather than an accident. A value too large
//! for `f32` becomes infinity, so it is rejected instead.

use crate::value::Value;

use super::super::error::DecodeError;
use super::mismatch;

/// Encode a `bool`: one byte, `0` or `1`.
///
/// # Arguments
///
/// * `value` — a [`Value::Bool`].
///
/// # Returns
///
/// A single byte.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for anything but a bool. Numbers are not coerced:
/// PostgreSQL's own binary `bool` admits only these two bytes.
pub(super) fn boolean(value: &Value) -> Result<Vec<u8>, DecodeError> {
    match value {
        Value::Bool(true) => Ok(vec![1]),
        Value::Bool(false) => Ok(vec![0]),
        other => Err(mismatch("bool", other)),
    }
}

/// Encode a `float4`: 4 big-endian IEEE-754 bytes.
///
/// # Arguments
///
/// * `value` — a [`Value::Float`] or [`Value::Int`].
///
/// # Returns
///
/// 4 bytes, big-endian.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-numeric value, or for a finite input whose
/// magnitude overflows `f32` — sending infinity for a real number the caller passed
/// would be a silent corruption.
pub(super) fn float4(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let wide = require_float(value, "float4")?;
    let narrow = wide as f32;
    if wide.is_finite() && !narrow.is_finite() {
        return Err(DecodeError::BadValue {
            what: "float4",
            detail: format!("{wide} overflows float4; bind it as float8 or numeric"),
        });
    }
    Ok(narrow.to_bits().to_be_bytes().to_vec())
}

/// Encode a `float8`: 8 big-endian IEEE-754 bytes.
///
/// # Arguments
///
/// * `value` — a [`Value::Float`] or [`Value::Int`].
///
/// # Returns
///
/// 8 bytes, big-endian.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-numeric value.
pub(super) fn float8(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let wide = require_float(value, "float8")?;
    Ok(wide.to_bits().to_be_bytes().to_vec())
}

/// Extract an `f64`, widening an `Int`, or report the mismatch by name.
fn require_float(value: &Value, what: &'static str) -> Result<f64, DecodeError> {
    match value {
        Value::Float(float) => Ok(*float),
        // Widening an integer literal into a float column is unambiguous.
        Value::Int(int) => Ok(*int as f64),
        other => Err(mismatch(what, other)),
    }
}
