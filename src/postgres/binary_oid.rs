//! # PostgreSQL type OIDs
//!
//! Built-in type OIDs are stable across releases and are baked into
//! `pg_type.dat` in the server source, so hard-coding them needs no catalogue
//! round trip. Values here are transcribed from `catalog/pg_type.dat`.
//!
//! Array types have their own OIDs, distinct from their element type. The array
//! constants and the [`element_of`] mapping live in `binary_oid_array.rs` and are
//! re-exported here so callers see one flat namespace.
//!
//! Only these OIDs get a binary decoder — anything else is reported as
//! [`super::DecodeError::UnsupportedOid`] and read as text instead.

/// `bool`, one byte: 0 or 1.
pub const BOOL: u32 = 16;
/// `bytea`, raw bytes with no framing.
pub const BYTEA: u32 = 17;
/// `char` (single-byte internal type), decoded as text.
pub const CHAR: u32 = 18;
/// `name`, a 63-byte identifier string.
pub const NAME: u32 = 19;
/// `int8`, 8-byte big-endian signed.
pub const INT8: u32 = 20;
/// `int2`, 2-byte big-endian signed.
pub const INT2: u32 = 21;
/// `int4`, 4-byte big-endian signed.
pub const INT4: u32 = 23;
/// `text`, UTF-8 with no length prefix inside the field.
pub const TEXT: u32 = 25;
/// `oid`, transported as a 4-byte unsigned value.
pub const OID: u32 = 26;
/// `json`, UTF-8 document with **no** version byte.
pub const JSON: u32 = 114;
/// `xml`, treated as text.
pub const XML: u32 = 142;
/// `float4`, big-endian IEEE-754 single.
pub const FLOAT4: u32 = 700;
/// `float8`, big-endian IEEE-754 double.
pub const FLOAT8: u32 = 701;
/// `bpchar` (blank-padded `char(n)`), decoded as text.
pub const BPCHAR: u32 = 1042;
/// `varchar`, decoded as text.
pub const VARCHAR: u32 = 1043;
/// `date`, 4-byte big-endian **days since 2000-01-01**.
pub const DATE: u32 = 1082;
/// `time`, 8-byte big-endian microseconds since midnight.
pub const TIME: u32 = 1083;
/// `timestamp`, 8-byte big-endian **microseconds since 2000-01-01T00:00:00**.
pub const TIMESTAMP: u32 = 1114;
/// `timestamptz`, same layout as [`TIMESTAMP`], always UTC on the wire.
pub const TIMESTAMPTZ: u32 = 1184;
/// `numeric`, base-10000 digit groups; never routed through `f64`.
pub const NUMERIC: u32 = 1700;
/// `uuid`, 16 raw bytes in network order.
pub const UUID: u32 = 2950;
/// `jsonb`, UTF-8 document behind a **leading version byte** (`1`).
pub const JSONB: u32 = 3802;

#[path = "binary_oid_array.rs"]
mod array;

// Array OID constants are the module's public surface; nothing internal names them yet.
#[allow(unused_imports)]
pub use array::{
    element_of, BOOL_ARRAY, BYTEA_ARRAY, DATE_ARRAY, FLOAT4_ARRAY, FLOAT8_ARRAY, INT2_ARRAY,
    INT4_ARRAY, INT8_ARRAY, JSONB_ARRAY, JSON_ARRAY, NUMERIC_ARRAY, TEXT_ARRAY, TIMESTAMPTZ_ARRAY,
    TIMESTAMP_ARRAY, TIME_ARRAY, UUID_ARRAY, VARCHAR_ARRAY,
};
