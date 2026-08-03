//! Coercing a script argument into a Redis **key or value** byte string.
//!
//! One concern: getting bytes out of a [`Value`] without losing or mangling any.
//!
//! # Why CRLF in a key or value is safe
//!
//! RESP requests are length-prefixed arrays, never inline text. [`encode_command`]
//! writes `$<len>\r\n<bytes>\r\n`, so the server reads exactly `len` bytes and
//! never scans the payload for a delimiter. A key such as `"a\r\nFLUSHALL\r\n"` is
//! therefore one 15-byte argument, not three commands: there is no delimiter for
//! it to escape, so there is nothing to escape *from*. The only client that could
//! corrupt such a payload is one that builds requests by concatenating text, which
//! is exactly what this layer refuses to do.
//!
//! Bytes are consequently carried as `Vec<u8>` end to end, and [`Value::Bytes`] is
//! accepted alongside [`Value::Str`] so a render cache can store a PNG.
//!
//! [`encode_command`]: crate::redis::encode_command

use super::args_error;
use crate::value::Value;

/// Coerce an argument into raw bytes for the wire.
///
/// # Arguments
///
/// * `method` — Fully qualified method, for the error message.
/// * `parameter` — Parameter name, for the error message.
/// * `value` — The supplied argument.
///
/// # Returns
///
/// The bytes, byte-for-byte. A `str` contributes its UTF-8 encoding; a `bytes`
/// contributes its contents unchanged, including NUL and CRLF.
///
/// # Errors
///
/// Returns [`args_error::mismatch`] naming `parameter` and the actual type for
/// anything else. An `int` is refused rather than stringified: silently accepting
/// `1` as `"1"` would make `redis.get(1)` and `redis.get("1")` the same key, hiding
/// a real bug in the caller.
///
/// # Examples
///
/// ```rust
/// use std::rc::Rc;
/// use tetherscript::redis_cap::coerce_bytes;
/// use tetherscript::value::Value;
///
/// let key = Value::Str(Rc::new("session:42".to_string()));
/// assert_eq!(coerce_bytes::bytes("redis.get", "key", &key).unwrap(), b"session:42");
///
/// // A CRLF-bearing key survives intact; it is one length-prefixed argument.
/// let hostile = Value::Str(Rc::new("a\r\nFLUSHALL\r\n".to_string()));
/// let raw = coerce_bytes::bytes("redis.get", "key", &hostile).unwrap();
/// assert_eq!(raw.len(), 13);
///
/// // An int is refused, naming the parameter.
/// let error = coerce_bytes::bytes("redis.get", "key", &Value::Int(1)).unwrap_err();
/// assert!(error.contains("`key`"));
/// ```
pub fn bytes(method: &str, parameter: &str, value: &Value) -> Result<Vec<u8>, String> {
    match value {
        Value::Str(text) => Ok(text.as_bytes().to_vec()),
        Value::Bytes(raw) => Ok(raw.borrow().clone()),
        other => Err(args_error::mismatch(
            method,
            parameter,
            "a str or bytes",
            other,
        )),
    }
}
