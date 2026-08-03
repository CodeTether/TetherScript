//! # Date/time parameter encoders
//!
//! Accepts either an ISO-8601 string — the same form the decoders produce, so a value
//! read from one query can be bound straight into the next — or an integer, which is
//! taken as the PostgreSQL counter already in its native units.
//!
//! ## The epoch shift runs in reverse here
//!
//! Encoding subtracts what decoding added: a Unix instant becomes
//! `unix_micros - PG_EPOCH_UNIX_MICROS`, and a civil date becomes
//! `unix_days - PG_EPOCH_UNIX_DAYS`. Skipping it sends a value 30 years off, and the
//! server accepts it happily because it is a perfectly legal timestamp. See
//! [`PG_EPOCH_UNIX_SECONDS`](crate::postgres::binary::PG_EPOCH_UNIX_SECONDS) for the
//! constant and its derivation.
//!
//! ## Why an integer input is the raw counter, not a Unix timestamp
//!
//! An integer here is ambiguous by nature, so it is defined as the value PostgreSQL
//! itself uses — microseconds since 2000-01-01 for `timestamp`, days since
//! 2000-01-01 for `date`. That makes `encode` the exact inverse of the wire read, and
//! anyone who has a Unix timestamp should pass the ISO string instead, which is
//! unambiguous. Guessing between the two based on magnitude would be worse than
//! either choice.

use crate::value::Value;

use super::super::error::DecodeError;
use super::super::time::{PG_EPOCH_UNIX_DAYS, PG_EPOCH_UNIX_MICROS};
use super::mismatch;

#[path = "binary_encode_iso.rs"]
mod iso;

/// Encode a `date`: 4 big-endian bytes, days since **2000-01-01**.
///
/// # Arguments
///
/// * `value` — `YYYY-MM-DD` as a [`Value::Str`], or a [`Value::Int`] already holding
///   days since 2000-01-01.
///
/// # Returns
///
/// 4 bytes, big-endian.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for an unparsable string, an out-of-`i32` day count, or
/// a value that is neither a string nor an int.
pub(super) fn date(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let pg_days = match value {
        Value::Int(days) => i32::try_from(*days).map_err(|_| far("date", *days))?,
        Value::Str(text) => {
            let unix_days = iso::parse_date(text)?;
            let shifted = unix_days - PG_EPOCH_UNIX_DAYS as i64;
            i32::try_from(shifted).map_err(|_| far("date", shifted))?
        }
        other => return Err(mismatch("date", other)),
    };
    Ok(pg_days.to_be_bytes().to_vec())
}

/// Encode a `time`: 8 big-endian bytes, microseconds since midnight.
///
/// # Arguments
///
/// * `value` — `HH:MM:SS[.ffffff]` as a [`Value::Str`], or a [`Value::Int`] of
///   microseconds since midnight.
///
/// # Returns
///
/// 8 bytes, big-endian.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for an unparsable string or a wrong value kind.
pub(super) fn time(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let micros = match value {
        Value::Int(micros) => *micros,
        Value::Str(text) => iso::parse_time(text)?,
        other => return Err(mismatch("time", other)),
    };
    Ok(micros.to_be_bytes().to_vec())
}

/// Encode a `timestamp` or `timestamptz`: microseconds since **2000-01-01**.
///
/// # Arguments
///
/// * `value` — `YYYY-MM-DD[T ]HH:MM:SS[.ffffff][Z]` as a [`Value::Str`], or a
///   [`Value::Int`] already holding microseconds since 2000-01-01.
///
/// # Returns
///
/// 8 bytes, big-endian.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for an unparsable string or a wrong value kind.
///
/// A trailing `Z` is accepted and ignored: `timestamptz` is always UTC on the wire,
/// so there is no offset field to fill in, and a non-UTC offset would need a zone
/// database this crate deliberately does not carry.
pub(super) fn timestamp(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let pg_micros = match value {
        Value::Int(micros) => *micros,
        Value::Str(text) => {
            let unix_micros = iso::parse_timestamp(text)?;
            // Reverse of the decode shift; forgetting it lands 30 years off.
            unix_micros.saturating_sub(PG_EPOCH_UNIX_MICROS)
        }
        other => return Err(mismatch("timestamp", other)),
    };
    Ok(pg_micros.to_be_bytes().to_vec())
}

/// Report a counter that will not fit its wire width.
fn far(what: &'static str, value: i64) -> DecodeError {
    DecodeError::BadValue {
        what,
        detail: format!("{value} is outside the range representable as {what}"),
    }
}
