//! Coercing a variadic key list.
//!
//! One concern: turning `arguments` into owned byte vectors, naming the *position*
//! of whichever one is wrong. Split from `super::methods_key` so that module owns
//! only the two commands.
//!
//! The keys are collected into owned `Vec<u8>` first and only then borrowed as
//! `&[&[u8]]` for [`Connection::del`](crate::redis::Connection::del), because a
//! `Value::Bytes` payload lives behind a
//! [`RefCell`](std::cell::RefCell) and cannot be borrowed for the length of the
//! call.

use crate::redis_cap::coerce_bytes;
use crate::value::Value;

/// Coerce every argument into key bytes.
///
/// # Arguments
///
/// * `method` — Fully qualified method, for the error message.
/// * `arguments` — One or more keys.
///
/// # Returns
///
/// The keys as owned byte vectors, in order.
///
/// # Errors
///
/// Returns the first mismatch, naming the parameter as `key[<n>]` so a caller with
/// six keys learns *which* one is wrong rather than just that one is.
///
/// # Examples
///
/// ```rust
/// use std::rc::Rc;
/// use tetherscript::redis_cap::methods_key_args;
/// use tetherscript::value::Value;
///
/// let supplied = [Value::Str(Rc::new("a".to_string())), Value::Int(2)];
/// let error = methods_key_args::collect("redis.del", &supplied).unwrap_err();
/// assert!(error.contains("key[1]"), "got: {error}");
/// ```
pub fn collect(method: &str, arguments: &[Value]) -> Result<Vec<Vec<u8>>, String> {
    arguments
        .iter()
        .enumerate()
        .map(|(index, value)| coerce_bytes::bytes(method, &format!("key[{index}]"), value))
        .collect()
}
