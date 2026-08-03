//! # Type-driven decoding of one `DataRow` field
//!
//! Entry point for the binary path. A `DataRow` field is a 4-byte big-endian
//! length followed by that many bytes, where **-1 means SQL NULL** and carries no
//! bytes at all. That is *not* the same as a length of 0, which is a present,
//! empty value: conflating them turns `''` into `nil` and vice versa, so the two
//! cases are separated at the type level here — [`decode_nullable`] takes
//! `Option<&[u8]>`, and `None` is the only thing that becomes [`Value::Nil`].
//!
//! ## Fallback contract for the integrator
//!
//! An OID with no binary decoder produces [`DecodeError::UnsupportedOid`], for
//! which [`DecodeError::needs_text_fallback`] is `true`. Callers must treat that
//! as "re-read this column as text", never as a query failure — otherwise adding
//! a column of an exotic type to one table breaks every route that selects it.
//! [`supports`] answers the same question up front, which lets a caller choose the
//! per-column format code *before* sending `Bind`.

use crate::value::Value;

use super::error::DecodeError;
use super::oid;

#[path = "binary_decode_array.rs"]
pub mod array;
#[path = "binary_decode_int.rs"]
mod int;
#[path = "binary_decode_numeric.rs"]
pub mod numeric;
#[path = "binary_decode_real.rs"]
mod real;
#[path = "binary_decode_scalar.rs"]
mod scalar;
#[path = "binary_decode_temporal.rs"]
mod temporal;
#[path = "binary_decode_text.rs"]
mod text;

/// Decode a present (non-NULL) binary field body.
///
/// # Arguments
///
/// * `type_oid` — the column's type OID from the `RowDescription`.
/// * `body` — the field bytes, already separated from the 4-byte length prefix.
///
/// # Returns
///
/// The field as a tetherscript [`Value`]. Array OIDs yield a
/// [`Value::List`]; `numeric` yields a [`Value::Str`] holding the exact decimal.
///
/// # Errors
///
/// - [`DecodeError::UnsupportedOid`] when no binary decoder exists — **recover by
///   decoding as text.**
/// - [`DecodeError::Truncated`] / [`DecodeError::Overlong`] when the body does not
///   match the type's layout.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{decode_field, oid};
/// use tetherscript::value::Value;
///
/// assert_eq!(decode_field(oid::BOOL, &[1]).unwrap(), Value::Bool(true));
/// assert_eq!(decode_field(oid::INT8, &[0, 0, 0, 0, 0, 0, 0, 7]).unwrap(), Value::Int(7));
/// assert!(decode_field(999_999, &[0]).unwrap_err().needs_text_fallback());
/// ```
pub fn decode_field(type_oid: u32, body: &[u8]) -> Result<Value, DecodeError> {
    match oid::element_of(type_oid) {
        Some(element) => array::decode_array(element, body),
        None => scalar::decode(type_oid, body),
    }
}

/// Decode a field that may be SQL NULL.
///
/// # Arguments
///
/// * `type_oid` — the column's type OID.
/// * `field` — `None` for a wire length of -1 (SQL NULL), `Some(bytes)` otherwise.
///   `Some(&[])` is a zero-length **present** value, not NULL.
///
/// # Returns
///
/// [`Value::Nil`] for `None`, otherwise whatever [`decode_field`] produces.
///
/// # Errors
///
/// Same as [`decode_field`]. A NULL never fails, whatever its declared type.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{decode_nullable, oid};
/// use tetherscript::value::Value;
///
/// // NULL and the empty string are different values.
/// assert_eq!(decode_nullable(oid::TEXT, None).unwrap(), Value::Nil);
/// assert_eq!(
///     decode_nullable(oid::TEXT, Some(&[])).unwrap(),
///     Value::Str(std::rc::Rc::new(String::new()))
/// );
/// ```
pub fn decode_nullable(type_oid: u32, field: Option<&[u8]>) -> Result<Value, DecodeError> {
    match field {
        None => Ok(Value::Nil),
        Some(body) => decode_field(type_oid, body),
    }
}

/// Whether a binary decoder exists for `type_oid`.
///
/// # Arguments
///
/// * `type_oid` — a scalar or array type OID.
///
/// # Returns
///
/// `true` when [`decode_field`] will not report
/// [`DecodeError::UnsupportedOid`], so the caller may request format code 1 for
/// that column in `Bind`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{oid, supports};
///
/// assert!(supports(oid::TIMESTAMPTZ));
/// assert!(supports(oid::INT4_ARRAY));
/// assert!(!supports(600)); // point: text only
/// ```
pub fn supports(type_oid: u32) -> bool {
    match oid::element_of(type_oid) {
        Some(element) => scalar::supports(element),
        None => scalar::supports(type_oid),
    }
}
