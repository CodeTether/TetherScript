//! # Scalar dispatch: type OID to binary decoder
//!
//! One `match` mapping a type OID to the decoder that owns its layout, plus the
//! [`supports`] predicate that answers the same question without decoding. Keeping
//! dispatch here — and the layouts in `int`, `real`, `text`, `temporal`,
//! `numeric` — means adding a type touches exactly two places.
//!
//! The fallthrough arm is the load-bearing part: an unrecognised OID yields
//! [`DecodeError::UnsupportedOid`], which the caller converts into a text-format
//! re-read. It must never become a hard failure, or adding a column of an exotic
//! type to one table would break every route that selects it.

use crate::value::Value;

use super::super::error::DecodeError;
use super::super::oid;
use super::{int, numeric, real, temporal, text};

/// Decode a non-array binary field body.
///
/// # Arguments
///
/// * `type_oid` — a scalar type OID.
/// * `body` — the field bytes without the length prefix.
///
/// # Returns
///
/// The decoded [`Value`].
///
/// # Errors
///
/// [`DecodeError::UnsupportedOid`] for an unregistered OID (recover as text), or a
/// layout error from the individual decoder.
pub(super) fn decode(type_oid: u32, body: &[u8]) -> Result<Value, DecodeError> {
    match type_oid {
        oid::BOOL => real::boolean(body),
        oid::FLOAT4 => real::float4(body),
        oid::FLOAT8 => real::float8(body),
        oid::INT2 => int::int2(body),
        oid::INT4 => int::int4(body),
        oid::INT8 => int::int8(body),
        oid::OID => int::oid_value(body),
        oid::NUMERIC => numeric::decode(body),
        oid::TEXT | oid::VARCHAR | oid::BPCHAR | oid::CHAR | oid::NAME | oid::XML => {
            text::utf8(body, "text")
        }
        oid::JSON => text::utf8(body, "json"),
        oid::JSONB => text::jsonb(body),
        oid::BYTEA => Ok(text::bytea(body)),
        oid::UUID => text::uuid(body),
        oid::DATE => temporal::date(body),
        oid::TIME => temporal::time(body),
        oid::TIMESTAMP => temporal::timestamp(body, false),
        oid::TIMESTAMPTZ => temporal::timestamp(body, true),
        unknown => Err(DecodeError::UnsupportedOid { oid: unknown }),
    }
}

/// Whether [`decode`] has a decoder for `type_oid`.
///
/// # Arguments
///
/// * `type_oid` — a scalar type OID.
///
/// # Returns
///
/// `true` when the OID will not produce [`DecodeError::UnsupportedOid`].
///
/// Probed by decoding an empty body: every real decoder rejects that as truncated,
/// and only the fallthrough arm reports an unsupported OID, so the two are
/// distinguishable without a second table that could drift out of sync with the
/// `match` above. `bytea` accepts an empty body, which is also not an unsupported
/// OID, so the predicate stays correct.
pub(super) fn supports(type_oid: u32) -> bool {
    !matches!(
        decode(type_oid, &[]),
        Err(DecodeError::UnsupportedOid { .. })
    )
}
