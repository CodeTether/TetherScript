//! # Typed binary parameter encoding
//!
//! Today `src/postgres/params.rs` sends every parameter in **text** format with an
//! unspecified type OID, so the server infers types from the statement. That is
//! safe but imprecise: a value bound against a `timestamptz` column needs an
//! explicit cast, and a decimal round-trips through a float literal.
//!
//! This module encodes the same OID set the decoder handles, so a caller can bind an
//! **exact** type. Every multi-byte value is written **big-endian** with
//! `to_be_bytes`, matching the protocol.
//!
//! ## How a caller requests binary format
//!
//! Format is negotiated per value in the `Bind` message, which carries two
//! independent format-code arrays:
//!
//! ```text
//! Bind:
//!   cstr    destination portal
//!   cstr    source statement
//!   int16   parameter format-code count    <-- 0 = all text, 1 = one code for all,
//!   int16[] parameter format codes             n = one code per parameter
//!   int16   parameter count
//!   for each: int32 length (-1 = NULL) + bytes
//!   int16   result-column format-code count  <-- same three-way convention
//!   int16[] result-column format codes
//! ```
//!
//! Code `0` is text and `1` is binary. So to *send* binary parameters, emit
//! [`FORMAT_BINARY`](crate::postgres::binary::FORMAT_BINARY) in the parameter array;
//! to *receive* binary rows, emit it in the result array.
//! [`format_codes`](crate::postgres::binary::format_codes) builds either array with
//! the right count word. The type OID itself is declared separately, in `Parse`. See
//! `binary_bind.rs` for the full integrator recipe.
//!
//! ## Errors here are caller errors, not wire errors
//!
//! An encoding failure means the *program* asked for something impossible — binding
//! a list to an `int4`, or a malformed UUID string. Those reuse
//! [`DecodeError::BadValue`] so the integrator has a single error type to convert.

use crate::value::Value;

use super::error::DecodeError;

#[path = "binary_encode_array.rs"]
mod array;
#[path = "binary_encode_dispatch.rs"]
mod dispatch;
#[path = "binary_encode_int.rs"]
mod int;
#[path = "binary_encode_numeric.rs"]
mod numeric;
#[path = "binary_encode_real.rs"]
mod real;
#[path = "binary_encode_temporal.rs"]
mod temporal;
#[path = "binary_encode_text.rs"]
mod text;

/// Encode a tetherscript value as a binary parameter for a declared type OID.
///
/// # Arguments
///
/// * `type_oid` — the OID to declare in `Parse` and to encode for.
/// * `value` — the value to bind. [`Value::Nil`] is SQL NULL.
///
/// # Returns
///
/// `Ok(None)` for SQL NULL, which the caller writes as a length of **-1 with no
/// bytes**. `Ok(Some(bytes))` otherwise, ready to be written after its own 4-byte
/// big-endian length. `Some(vec![])` is a legitimate zero-length value — an empty
/// string or empty `bytea` — and must **not** be collapsed into NULL.
///
/// # Errors
///
/// - [`DecodeError::UnsupportedOid`] when no binary encoder exists for `type_oid`;
///   the caller falls back to text format for that one parameter.
/// - [`DecodeError::BadValue`] when the value cannot represent that type, naming
///   both the type and the value's kind.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{encode_param, oid};
/// use tetherscript::value::Value;
///
/// // int4 42, big-endian.
/// assert_eq!(encode_param(oid::INT4, &Value::Int(42)).unwrap(), Some(vec![0, 0, 0, 42]));
/// // NULL is None here, and a -1 length on the wire.
/// assert_eq!(encode_param(oid::INT4, &Value::Nil).unwrap(), None);
/// // An empty string is a present, zero-length value, distinct from NULL.
/// assert_eq!(
///     encode_param(oid::TEXT, &Value::Str(std::rc::Rc::new(String::new()))).unwrap(),
///     Some(vec![])
/// );
/// ```
pub fn encode_param(type_oid: u32, value: &Value) -> Result<Option<Vec<u8>>, DecodeError> {
    if matches!(value, Value::Nil) {
        // Nil is NULL for every type, so this precedes type dispatch entirely.
        return Ok(None);
    }
    dispatch::encode(type_oid, value).map(Some)
}

/// Build the `BadValue` error used across the encoders.
///
/// # Arguments
///
/// * `what` — the SQL type name being encoded for.
/// * `value` — the offending value, whose type name is reported.
///
/// # Returns
///
/// A [`DecodeError::BadValue`] naming both sides of the mismatch, so the message
/// says what was expected and what arrived rather than just "bad parameter".
pub(super) fn mismatch(what: &'static str, value: &Value) -> DecodeError {
    DecodeError::BadValue {
        what,
        detail: format!("cannot bind a {} value as {what}", value.type_name()),
    }
}
