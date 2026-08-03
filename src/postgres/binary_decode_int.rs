//! # Fixed-width integer decoders
//!
//! Every integer type here has an exact byte length, so the decoder reads that
//! many **big-endian** bytes and then calls `finish` to reject a longer body. The
//! `finish` check matters: a `bytea` mistakenly labelled `int4` would otherwise
//! decode its first four bytes and silently produce a number.
//!
//! | Type | Bytes | Layout |
//! |---|---|---|
//! | `int2` | 2 | big-endian two's complement |
//! | `int4` | 4 | big-endian two's complement |
//! | `int8` | 8 | big-endian two's complement |
//! | `oid` | 4 | big-endian **unsigned** |
//!
//! `int2`/`int4` widen into tetherscript's `i64` `Int` losslessly. `oid` is
//! unsigned despite sharing `int4`'s width, so it is reinterpreted through `u32` —
//! otherwise an OID above 2^31 would decode negative.

use crate::value::Value;

use super::super::error::DecodeError;
use super::super::read::Reader;

/// Decode an `int2` into an `i64` `Int`. Widening is lossless.
///
/// # Arguments
///
/// * `body` — exactly 2 big-endian bytes.
///
/// # Returns
///
/// [`Value::Int`].
///
/// # Errors
///
/// [`DecodeError::Truncated`] or [`DecodeError::Overlong`].
pub(super) fn int2(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let value = reader.i16("int2")?;
    reader.finish("int2")?;
    Ok(Value::Int(value as i64))
}

/// Decode an `int4` into an `i64` `Int`.
///
/// # Arguments
///
/// * `body` — exactly 4 big-endian bytes.
///
/// # Returns
///
/// [`Value::Int`].
///
/// # Errors
///
/// [`DecodeError::Truncated`] or [`DecodeError::Overlong`].
pub(super) fn int4(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let value = reader.i32("int4")?;
    reader.finish("int4")?;
    Ok(Value::Int(value as i64))
}

/// Decode an `int8`.
///
/// # Arguments
///
/// * `body` — exactly 8 big-endian bytes.
///
/// # Returns
///
/// [`Value::Int`].
///
/// # Errors
///
/// [`DecodeError::Truncated`] or [`DecodeError::Overlong`].
pub(super) fn int8(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let value = reader.i64("int8")?;
    reader.finish("int8")?;
    Ok(Value::Int(value))
}

/// Decode an `oid`, which is **unsigned** 32-bit.
///
/// # Arguments
///
/// * `body` — exactly 4 big-endian bytes.
///
/// # Returns
///
/// [`Value::Int`] in 0..=4_294_967_295, never negative.
///
/// # Errors
///
/// [`DecodeError::Truncated`] or [`DecodeError::Overlong`].
pub(super) fn oid_value(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let value = reader.i32("oid")? as u32;
    reader.finish("oid")?;
    Ok(Value::Int(value as i64))
}
