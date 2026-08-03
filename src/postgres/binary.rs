//! # Binary-format wire codec for the native PostgreSQL client
//!
//! The client historically read every field in **text** format, so a script got
//! strings it had to re-parse and an exact SQL type needed a cast in every query.
//! This module adds the binary path: type-driven decoding of a `DataRow` field and
//! type-driven encoding of a `Bind` parameter.
//!
//! ## Endianness — this is the whole ballgame
//!
//! **Every multi-byte scalar in the PostgreSQL v3 protocol is big-endian (network
//! byte order), without exception.** That covers message lengths, field lengths,
//! `int2`/`int4`/`int8`, the IEEE-754 bit patterns of `float4`/`float8`, every word
//! of a `numeric`, the timestamp microsecond counter, and every field of an array
//! header. Reading any of them with `from_le_bytes` on a little-endian host does not
//! fail loudly — it yields a byte-swapped number, so `0x00010203` reads as
//! `50462976` instead of `66051`. All reads therefore go through [`Reader`], which
//! only ever calls `from_be_bytes`, and all writes use `to_be_bytes`.
//!
//! ## Epochs — the other thing that silently produces wrong data
//!
//! PostgreSQL's binary `timestamp`/`timestamptz` is **microseconds since
//! 2000-01-01T00:00:00Z**, and binary `date` is **days since 2000-01-01**. Neither is
//! a Unix timestamp. Treating the microsecond counter as Unix seconds, or forgetting
//! the 30-year shift, yields dates that are wrong but well-formed — the worst kind of
//! bug. The shift constants are [`PG_EPOCH_UNIX_SECONDS`], [`PG_EPOCH_UNIX_MICROS`],
//! and [`PG_EPOCH_UNIX_DAYS`], each pinned by a test in `tests/postgres_binary.rs`.
//!
//! ## Safety posture
//!
//! A decoder here reads bytes supplied by a network peer, so **it must not panic on
//! any input**, of any length, including empty. Every length is checked before it is
//! trusted and every failure is a named [`DecodeError`]. There is no indexing,
//! slicing, or `unwrap` on an untrusted length anywhere in the module.
//!
//! ## Unknown type OIDs fall back to text
//!
//! [`decode_field`] returns [`DecodeError::UnsupportedOid`] — for which
//! [`DecodeError::needs_text_fallback`] is `true` — instead of failing the query. The
//! integrator uses that predicate to keep a column of an unregistered type readable
//! as text, so adding a type to a table never breaks an unrelated route. [`supports`]
//! answers the same question up front, which lets a caller pick the per-column format
//! code *before* sending `Bind`.
//!
//! ## Quick start
//!
//! ```rust
//! use tetherscript::postgres::binary::{decode_field, decode_nullable, oid};
//! use tetherscript::value::Value;
//!
//! // int4 66051, big-endian. Little-endian would read 50462976.
//! assert_eq!(decode_field(oid::INT4, &[0, 1, 2, 3]).unwrap(), Value::Int(66_051));
//!
//! // A field length of -1 is SQL NULL, which is not the same as a 0-length value.
//! assert_eq!(decode_nullable(oid::TEXT, None).unwrap(), Value::Nil);
//! ```

#[path = "binary_bind.rs"]
mod bind;
#[path = "binary_decode.rs"]
mod decode;
#[path = "binary_encode.rs"]
mod encode;
#[path = "binary_error.rs"]
mod error;
#[path = "binary_oid.rs"]
pub mod oid;
#[path = "binary_read.rs"]
mod read;
#[path = "binary_time.rs"]
mod time;
#[path = "binary_uuid.rs"]
mod uuid;

pub use bind::{FORMAT_BINARY, FORMAT_TEXT, format_codes};
pub use decode::array::decode_array;
pub use decode::numeric::numeric_to_string;
pub use decode::{decode_field, decode_nullable, supports};
pub use encode::encode_param;
pub use error::DecodeError;
pub use read::Reader;
pub use time::civil::{civil_from_days, days_from_civil};
pub use time::{
    PG_EPOCH_UNIX_DAYS, PG_EPOCH_UNIX_MICROS, PG_EPOCH_UNIX_SECONDS, date_unix_days,
    timestamp_unix_micros,
};
