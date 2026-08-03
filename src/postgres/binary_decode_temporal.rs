//! # Date/time decoders: `date`, `time`, `timestamp`, `timestamptz`
//!
//! Each is a fixed-width **big-endian** integer counter. What makes them dangerous
//! is not the framing but the units and the epoch:
//!
//! | Type | Bytes | Unit | Epoch |
//! |---|---|---|---|
//! | `date` | 4 | days | **2000-01-01** |
//! | `time` | 8 | microseconds | midnight |
//! | `timestamp` | 8 | **microseconds** | **2000-01-01** |
//! | `timestamptz` | 8 | **microseconds** | **2000-01-01 UTC** |
//!
//! So a `timestamptz` is neither seconds nor Unix-based. Reading the counter as
//! Unix seconds lands in the year 24-million; reading microseconds but skipping the
//! 30-year shift lands in 1970 instead of 2000. Both produce well-formed dates,
//! which is why the shift is a named constant with a pinned test rather than an
//! inline literal — see [`super::super::time`].
//!
//! Values are rendered as ISO-8601 strings rather than integers. A script that
//! wanted the epoch integer could not tell a `timestamp` from an `int8`; a string
//! carries its own meaning, sorts correctly, and round-trips back into SQL. That
//! is the concrete fix for the workaround where timestamps had to be stored as
//! bigint Unix seconds because only text decoding existed.

use std::rc::Rc;

use crate::value::Value;

use super::super::error::DecodeError;
use super::super::read::Reader;

#[path = "binary_decode_iso.rs"]
mod iso;

/// Decode a `date`: 4 big-endian bytes, days since **2000-01-01**.
///
/// # Arguments
///
/// * `body` — exactly 4 bytes.
///
/// # Returns
///
/// [`Value::Str`] as `YYYY-MM-DD`, or `infinity`/`-infinity` for the sentinels.
///
/// # Errors
///
/// [`DecodeError::Truncated`] or [`DecodeError::Overlong`].
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{decode_field, oid};
/// use tetherscript::value::Value;
///
/// // 8780 days after 2000-01-01 is 2024-01-15.
/// let decoded = decode_field(oid::DATE, &8_780i32.to_be_bytes()).unwrap();
/// assert_eq!(decoded, Value::Str(std::rc::Rc::new("2024-01-15".into())));
/// ```
pub(super) fn date(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let days = reader.i32("date")?;
    reader.finish("date")?;
    Ok(Value::Str(Rc::new(iso::date(days))))
}

/// Decode a `time`: 8 big-endian bytes, microseconds since midnight.
///
/// # Arguments
///
/// * `body` — exactly 8 bytes.
///
/// # Returns
///
/// [`Value::Str`] as `HH:MM:SS` with a fractional part only when non-zero.
///
/// # Errors
///
/// [`DecodeError::Truncated`] or [`DecodeError::Overlong`].
pub(super) fn time(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let micros = reader.i64("time")?;
    reader.finish("time")?;
    Ok(Value::Str(Rc::new(iso::time_of_day(micros))))
}

/// Decode a `timestamp` or `timestamptz`: microseconds since **2000-01-01**.
///
/// # Arguments
///
/// * `body` — exactly 8 big-endian bytes.
/// * `utc` — append `Z`; `true` for `timestamptz`, which is always UTC on the wire
///   regardless of the session `TimeZone`, so binary decoding is zone-independent.
///
/// # Returns
///
/// [`Value::Str`] as `YYYY-MM-DDTHH:MM:SS[.ffffff][Z]`.
///
/// # Errors
///
/// [`DecodeError::Truncated`] or [`DecodeError::Overlong`].
pub(super) fn timestamp(body: &[u8], utc: bool) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let micros = reader.i64("timestamp")?;
    reader.finish("timestamp")?;
    Ok(Value::Str(Rc::new(iso::timestamp(micros, utc))))
}
