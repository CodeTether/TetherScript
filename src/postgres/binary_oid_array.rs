//! # Array type OIDs and the array-to-element mapping
//!
//! Each PostgreSQL array type has its own OID, unrelated numerically to its
//! element type, so the mapping has to be a table rather than arithmetic. The
//! wire body of an array carries the element OID in its header too, but the
//! *column's* own OID is what tells the decoder to expect an array at all — so
//! both are needed, and the decoder cross-checks them.
//!
//! Values transcribed from the server's `catalog/pg_type.dat` (`array_type_oid`).

/// `bool[]`
pub const BOOL_ARRAY: u32 = 1000;
/// `bytea[]`
pub const BYTEA_ARRAY: u32 = 1001;
/// `int2[]`
pub const INT2_ARRAY: u32 = 1005;
/// `int4[]`
pub const INT4_ARRAY: u32 = 1007;
/// `text[]`
pub const TEXT_ARRAY: u32 = 1009;
/// `varchar[]`
pub const VARCHAR_ARRAY: u32 = 1015;
/// `int8[]`
pub const INT8_ARRAY: u32 = 1016;
/// `float4[]`
pub const FLOAT4_ARRAY: u32 = 1021;
/// `float8[]`
pub const FLOAT8_ARRAY: u32 = 1022;
/// `json[]`
pub const JSON_ARRAY: u32 = 199;
/// `date[]`
pub const DATE_ARRAY: u32 = 1182;
/// `time[]`
pub const TIME_ARRAY: u32 = 1183;
/// `timestamp[]`
pub const TIMESTAMP_ARRAY: u32 = 1115;
/// `timestamptz[]`
pub const TIMESTAMPTZ_ARRAY: u32 = 1185;
/// `numeric[]`
pub const NUMERIC_ARRAY: u32 = 1231;
/// `uuid[]`
pub const UUID_ARRAY: u32 = 2951;
/// `jsonb[]`
pub const JSONB_ARRAY: u32 = 3807;

/// Element type OID for an array type OID.
///
/// # Arguments
///
/// * `array_oid` — the column's own type OID.
///
/// # Returns
///
/// `Some(element_oid)` when `array_oid` is a supported array type, and `None` when
/// it is not an array this codec handles — including every scalar OID, so callers
/// can use `None` to mean "decode as a scalar".
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::oid;
///
/// assert_eq!(oid::element_of(oid::INT4_ARRAY), Some(oid::INT4));
/// assert_eq!(oid::element_of(oid::TIMESTAMPTZ_ARRAY), Some(oid::TIMESTAMPTZ));
/// assert_eq!(oid::element_of(oid::INT4), None);
/// ```
pub fn element_of(array_oid: u32) -> Option<u32> {
    let element = match array_oid {
        BOOL_ARRAY => super::BOOL,
        BYTEA_ARRAY => super::BYTEA,
        INT2_ARRAY => super::INT2,
        INT4_ARRAY => super::INT4,
        INT8_ARRAY => super::INT8,
        TEXT_ARRAY => super::TEXT,
        VARCHAR_ARRAY => super::VARCHAR,
        FLOAT4_ARRAY => super::FLOAT4,
        FLOAT8_ARRAY => super::FLOAT8,
        JSON_ARRAY => super::JSON,
        JSONB_ARRAY => super::JSONB,
        DATE_ARRAY => super::DATE,
        TIME_ARRAY => super::TIME,
        TIMESTAMP_ARRAY => super::TIMESTAMP,
        TIMESTAMPTZ_ARRAY => super::TIMESTAMPTZ,
        NUMERIC_ARRAY => super::NUMERIC,
        UUID_ARRAY => super::UUID,
        _ => return None,
    };
    Some(element)
}
