//! # `bool`, `float4`, and `float8` decoders
//!
//! `bool` is a single byte and the floats are raw **big-endian** IEEE-754 bit
//! patterns, 4 and 8 bytes. All three call `finish` so a longer body is rejected
//! rather than silently truncated to a plausible value.
//!
//! `bool` accepts only `0` and `1`. PostgreSQL never sends anything else in binary
//! format, so tolerating a stray byte would hide a framing bug instead of
//! surfacing it.
//!
//! Note what is *not* here: `numeric` never passes through these functions.
//! Routing a decimal through `f64` is exactly the correctness bug `numeric` exists
//! to prevent — see `binary_decode_numeric.rs`.

use crate::value::Value;

use super::super::error::DecodeError;
use super::super::read::Reader;

/// Decode a `bool`: exactly one byte, `0` or `1`.
///
/// # Arguments
///
/// * `body` — the field bytes.
///
/// # Returns
///
/// [`Value::Bool`].
///
/// # Errors
///
/// [`DecodeError::Truncated`] on an empty body, [`DecodeError::Overlong`] on more
/// than one byte, and [`DecodeError::BadValue`] for any byte other than 0 or 1.
pub(super) fn boolean(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let byte = reader.take("bool", 1)?[0];
    reader.finish("bool")?;
    match byte {
        0 => Ok(Value::Bool(false)),
        1 => Ok(Value::Bool(true)),
        other => Err(DecodeError::BadValue {
            what: "bool",
            detail: format!("expected byte 0 or 1, got {other}"),
        }),
    }
}

/// Decode a `float4`, widened exactly to `f64`.
///
/// # Arguments
///
/// * `body` — exactly 4 big-endian bytes.
///
/// # Returns
///
/// [`Value::Float`].
///
/// # Errors
///
/// [`DecodeError::Truncated`] or [`DecodeError::Overlong`].
pub(super) fn float4(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let value = reader.f32("float4")?;
    reader.finish("float4")?;
    Ok(Value::Float(value))
}

/// Decode a `float8`.
///
/// # Arguments
///
/// * `body` — exactly 8 big-endian bytes.
///
/// # Returns
///
/// [`Value::Float`].
///
/// # Errors
///
/// [`DecodeError::Truncated`] or [`DecodeError::Overlong`].
pub(super) fn float8(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let value = reader.f64("float8")?;
    reader.finish("float8")?;
    Ok(Value::Float(value))
}
