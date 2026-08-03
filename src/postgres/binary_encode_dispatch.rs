//! # Encoder dispatch: type OID to binary encoder
//!
//! Mirror image of `binary_decode_scalar.rs`, kept as its own file so the encode
//! table and the decode table can be diffed against each other — a type present in
//! one and absent from the other is the bug this separation makes visible.
//!
//! The array arm delegates to [`array::encode`], which re-enters this dispatch for
//! each element, so `int4[]` needs no separate table entry beyond its OID mapping.
//!
//! The fallthrough is [`DecodeError::UnsupportedOid`]: the caller binds that one
//! parameter as text instead of failing the statement.

use crate::value::Value;

use super::super::error::DecodeError;
use super::super::oid;
use super::{array, int, numeric, real, temporal, text};

/// Encode `value` as binary for `type_oid`.
///
/// # Arguments
///
/// * `type_oid` — a scalar or array type OID.
/// * `value` — a non-nil value; NULL is handled before dispatch.
///
/// # Returns
///
/// The field bytes, without the 4-byte length prefix the caller writes.
///
/// # Errors
///
/// [`DecodeError::UnsupportedOid`] for an unregistered OID, or
/// [`DecodeError::BadValue`] when the value cannot represent that type.
pub(super) fn encode(type_oid: u32, value: &Value) -> Result<Vec<u8>, DecodeError> {
    match type_oid {
        oid::BOOL => real::boolean(value),
        oid::FLOAT4 => real::float4(value),
        oid::FLOAT8 => real::float8(value),
        oid::INT2 => int::int2(value),
        oid::INT4 => int::int4(value),
        oid::INT8 => int::int8(value),
        oid::OID => int::oid_value(value),
        oid::NUMERIC => numeric::encode(value),
        oid::TEXT | oid::VARCHAR | oid::BPCHAR | oid::CHAR | oid::NAME | oid::XML => {
            text::utf8(value, "text")
        }
        oid::JSON => text::utf8(value, "json"),
        oid::JSONB => text::jsonb(value),
        oid::BYTEA => text::bytea(value),
        oid::UUID => text::uuid(value),
        oid::DATE => temporal::date(value),
        oid::TIME => temporal::time(value),
        oid::TIMESTAMP | oid::TIMESTAMPTZ => temporal::timestamp(value),
        other => match oid::element_of(other) {
            Some(element) => array::encode(element, value),
            None => Err(DecodeError::UnsupportedOid { oid: other }),
        },
    }
}
