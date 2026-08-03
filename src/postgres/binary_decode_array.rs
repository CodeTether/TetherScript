//! # Binary array decoding
//!
//! ## Wire layout
//!
//! All words are **big-endian**:
//!
//! ```text
//! int32  ndim          number of dimensions
//! int32  has_null      1 if any element is NULL, else 0 (advisory only)
//! uint32 element_oid   type OID shared by every element
//! per dimension:
//!   int32 length       elements along this dimension
//!   int32 lower_bound  first subscript, normally 1
//! per element, in row-major order:
//!   int32 length       byte length, or -1 for a NULL element
//!   bytes value        omitted entirely when length is -1
//! ```
//!
//! ## Dimensions are rejected, not flattened
//!
//! Only `ndim` 0 (the empty array, which carries no dimension block at all) and 1
//! are decoded. A 2-D array is reported as
//! [`DecodeError::UnsupportedDimensions`] rather than flattened, because flattening
//! silently discards shape: `{{1,2},{3,4}}` would become `[1,2,3,4]` and a caller
//! could not tell. A negative or absurd `ndim` is rejected for the same reason it is
//! checked at all — it is an untrusted length driving a loop.
//!
//! ## A NULL element is length -1, not length 0
//!
//! Each element carries its own 4-byte length, and `-1` means NULL with **no value
//! bytes following**. A zero-length element is a present empty value. Conflating
//! them turns `{"",NULL}` into `{NULL,NULL}`. The distinction is preserved by passing
//! `None` versus `Some(&[])` into
//! [`decode_nullable`](crate::postgres::binary::decode_nullable).
//!
//! The `has_null` header flag is advisory: PostgreSQL sets it, but the per-element
//! length is authoritative, so this decoder never uses the flag to decide.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

use super::super::error::DecodeError;
use super::super::read::Reader;

#[path = "binary_decode_array_header.rs"]
mod header;

/// Decode a binary array body into a [`Value::List`].
///
/// # Arguments
///
/// * `element_oid` — element OID expected from the column's array OID, via
///   [`oid::element_of`](crate::postgres::binary::oid::element_of).
/// * `body` — the whole field body.
///
/// # Returns
///
/// [`Value::List`] of decoded elements, with a NULL element as [`Value::Nil`]. An
/// empty array yields an empty list.
///
/// # Errors
///
/// - [`DecodeError::UnsupportedDimensions`] for `ndim` outside 0..=1.
/// - [`DecodeError::BadValue`] when the header's element OID contradicts the
///   column's, or when a dimension length is negative.
/// - [`DecodeError::Truncated`] / [`DecodeError::Overlong`] on a malformed body.
/// - Whatever the element decoder reports, unchanged — including
///   [`DecodeError::UnsupportedOid`], so an array of an unregistered element type
///   also falls back to text.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{decode_field, oid};
/// use tetherscript::value::Value;
///
/// // int4[] holding {7, NULL}: ndim 1, has_null 1, elem oid 23, len 2, lower 1.
/// let body = [
///     0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 23, 0, 0, 0, 2, 0, 0, 0, 1,
///     0, 0, 0, 4, 0, 0, 0, 7, // element 0: 4 bytes, int4 7
///     255, 255, 255, 255, // element 1: length -1, SQL NULL, no bytes
/// ];
/// let decoded = decode_field(oid::INT4_ARRAY, &body).unwrap();
/// match decoded {
///     Value::List(items) => {
///         assert_eq!(items.borrow().len(), 2);
///         assert_eq!(items.borrow()[0], Value::Int(7));
///         assert_eq!(items.borrow()[1], Value::Nil);
///     }
///     other => panic!("expected a list, got {}", other.type_name()),
/// }
/// ```
pub fn decode_array(element_oid: u32, body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let count = header::parse(&mut reader, element_oid)?;
    let mut items = Vec::with_capacity(count.min(4_096));
    for _ in 0..count {
        items.push(element(&mut reader, element_oid)?);
    }
    reader.finish("array")?;
    Ok(Value::List(Rc::new(RefCell::new(items))))
}

/// Read one length-prefixed element. A length of -1 is SQL NULL with no bytes.
fn element(reader: &mut Reader<'_>, element_oid: u32) -> Result<Value, DecodeError> {
    let len = reader.i32("array element length")?;
    if len < 0 {
        // -1 is the only negative length the protocol defines, and it means NULL.
        return super::decode_nullable(element_oid, None);
    }
    let bytes = reader.take("array element", len as usize)?;
    super::decode_nullable(element_oid, Some(bytes))
}
