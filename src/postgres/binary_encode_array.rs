//! # Binary array parameter encoding
//!
//! Emits the same header the decoder reads, all words **big-endian**:
//!
//! ```text
//! int32 ndim (0 or 1), int32 has_null, uint32 element_oid
//! if ndim == 1: int32 length, int32 lower_bound (always 1)
//! per element: int32 length (-1 = NULL) + bytes
//! ```
//!
//! Three details that are easy to get wrong:
//!
//! - **An empty list emits `ndim = 0` and no dimension block at all.** Writing
//!   `ndim = 1` with `length = 0` is *not* the same message, and the server treats the
//!   two differently.
//! - **`lower_bound` is 1**, not 0. PostgreSQL arrays are 1-based, and sending 0
//!   produces an array whose first subscript is 0 — legal, and surprising to every
//!   query that reads it.
//! - **A nil element is length -1 with no bytes**, not a zero-length element. That is
//!   the same NULL-versus-empty distinction as a row field, one level down.
//!
//! `has_null` is computed from the elements rather than assumed, since the server does
//! read it, but the per-element lengths remain authoritative on both sides.

use crate::value::Value;

use super::super::error::DecodeError;
use super::{dispatch, mismatch};

/// Encode a list as a one-dimensional binary array of `element_oid`.
///
/// # Arguments
///
/// * `element_oid` — the element type OID, from
///   [`oid::element_of`](crate::postgres::binary::oid::element_of).
/// * `value` — a [`Value::List`]. Nested lists are rejected, matching the decoder's
///   refusal to flatten multi-dimensional arrays.
///
/// # Returns
///
/// The complete array field body, without the outer 4-byte length prefix.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-list value or a nested list, and whatever the
/// element encoder reports — including [`DecodeError::UnsupportedOid`], so an array of
/// an unregistered element type also falls back to text.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{decode_field, encode_param, oid};
/// use tetherscript::value::Value;
/// use std::{cell::RefCell, rc::Rc};
///
/// let list = Value::List(Rc::new(RefCell::new(vec![Value::Int(7), Value::Nil])));
/// let bytes = encode_param(oid::INT4_ARRAY, &list).unwrap().unwrap();
/// // Round-trips through the decoder, NULL element intact.
/// assert_eq!(decode_field(oid::INT4_ARRAY, &bytes).unwrap(), list);
/// ```
pub(super) fn encode(element_oid: u32, value: &Value) -> Result<Vec<u8>, DecodeError> {
    let items = match value {
        Value::List(items) => items.borrow().clone(),
        other => return Err(mismatch("array", other)),
    };
    let encoded = elements(element_oid, &items)?;
    let has_null = encoded.iter().any(Option::is_none);
    let mut out = header(element_oid, items.len(), has_null);
    for element in encoded {
        match element {
            // -1 length signals a NULL element and carries no value bytes.
            None => out.extend_from_slice(&(-1i32).to_be_bytes()),
            Some(bytes) => {
                out.extend_from_slice(&(bytes.len() as i32).to_be_bytes());
                out.extend_from_slice(&bytes);
            }
        }
    }
    Ok(out)
}

/// Encode each element, rejecting a nested list rather than flattening it.
fn elements(element_oid: u32, items: &[Value]) -> Result<Vec<Option<Vec<u8>>>, DecodeError> {
    items
        .iter()
        .map(|item| match item {
            Value::Nil => Ok(None),
            Value::List(_) => Err(DecodeError::BadValue {
                what: "array",
                detail: "nested lists would need a multi-dimensional array, which this \
                         codec does not encode"
                    .into(),
            }),
            other => dispatch::encode(element_oid, other).map(Some),
        })
        .collect()
}

/// Build the array header. An empty list gets `ndim = 0` and no dimension block.
fn header(element_oid: u32, len: usize, has_null: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(20);
    let ndim: i32 = if len == 0 { 0 } else { 1 };
    out.extend_from_slice(&ndim.to_be_bytes());
    out.extend_from_slice(&i32::from(has_null).to_be_bytes());
    out.extend_from_slice(&element_oid.to_be_bytes());
    if ndim == 1 {
        out.extend_from_slice(&(len as i32).to_be_bytes());
        // PostgreSQL arrays are 1-based; a 0 here shifts every subscript.
        out.extend_from_slice(&1i32.to_be_bytes());
    }
    out
}
