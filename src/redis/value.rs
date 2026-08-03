//! The RESP value model.
//!
//! One variant per RESP type, with two deliberate splits that a looser model
//! would collapse:
//!
//! - [`RespValue::NullBulk`] (`$-1\r\n`) is **not** [`RespValue::Bulk`] with an
//!   empty payload (`$0\r\n\r\n`). The first means *the key does not exist*; the
//!   second means *the key exists and holds the empty string*. A session store
//!   that treats them alike cannot tell a logged-out user from one whose session
//!   value happens to be empty, so they are separate variants and compare
//!   unequal.
//! - [`RespValue::NullArray`] (`*-1\r\n`) is likewise distinct from an empty
//!   array (`*0\r\n`); a blocking pop that timed out returns the former.
//!
//! Bulk payloads are `Vec<u8>`, not `String`: Redis strings are binary-safe and
//! may hold arbitrary bytes, including CRLF and invalid UTF-8.

/// A decoded RESP reply.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis::RespValue;
///
/// // A missing key and an empty value are different answers.
/// assert_ne!(RespValue::NullBulk, RespValue::Bulk(Vec::new()));
///
/// let nested = RespValue::Array(vec![
///     RespValue::Integer(1),
///     RespValue::Array(vec![RespValue::Bulk(b"inner".to_vec())]),
/// ]);
/// assert_eq!(nested.type_name(), "array");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RespValue {
    /// `+OK\r\n` — a status line. CRLF-free by construction.
    Simple(String),
    /// `-ERR ...\r\n` — an error reply, kept as a value so it can appear nested
    /// inside an array. The command layer promotes a top-level one to
    /// [`RedisError::Server`](super::RedisError::Server).
    Error {
        /// Leading token, such as `ERR` or `WRONGTYPE`.
        kind: String,
        /// Remainder of the error line.
        message: String,
    },
    /// `:42\r\n` — a 64-bit signed integer.
    Integer(i64),
    /// `$5\r\nhello\r\n` — a length-prefixed, binary-safe string.
    Bulk(Vec<u8>),
    /// `$-1\r\n` — the null bulk string: the value is absent.
    NullBulk,
    /// `*2\r\n...` — an array, possibly containing further arrays.
    Array(Vec<RespValue>),
    /// `*-1\r\n` — the null array: no result, as opposed to an empty one.
    NullArray,
}
