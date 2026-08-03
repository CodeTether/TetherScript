//! Key existence methods: `del` and `exists`.
//!
//! One concern: whole-key presence. Both are variadic in Redis and both answer with
//! a count, so they share a body; the capability exposes the multi-key form because
//! deleting a session and its index in one round trip is the normal case.

use crate::redis::Connection;
use crate::redis_cap::{args_missing, outcome};
use crate::value::Value;

/// `redis.del(key, ...)` — delete one or more keys.
///
/// # Arguments
///
/// * `arguments` — At least one key, each a `str` or `bytes`.
///
/// # Returns
///
/// `Ok(Result::Ok(n))`, the number of keys that existed and were removed. `0` is a
/// success, not an error: deleting an absent key is idempotent.
///
/// # Errors
///
/// A usage `Err` naming `key` when no key is supplied or one has the wrong type.
pub(super) fn del(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    counted(connection, "redis.del", arguments)
}

/// `redis.exists(key, ...)` — count how many of the keys exist.
///
/// # Returns
///
/// `Ok(Result::Ok(n))`. Kept an int rather than a bool so the multi-key form is
/// meaningful and the reply matches Redis, which counts a repeated key each time it
/// appears.
///
/// # Errors
///
/// As [`del`].
pub(super) fn exists(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    counted(connection, "redis.exists", arguments)
}

/// Shared body for the variadic, count-answering key commands.
///
/// # Errors
///
/// A usage `Err` for an empty argument list or a badly typed key.
fn counted(
    connection: &mut Connection,
    method: &str,
    arguments: &[Value],
) -> Result<Value, String> {
    if arguments.is_empty() {
        return Err(args_missing::missing(method, "key"));
    }
    let owned = super::methods_key_args::collect(method, arguments)?;
    let keys: Vec<&[u8]> = owned.iter().map(Vec::as_slice).collect();
    let sent = match method {
        "redis.del" => connection.del(&keys),
        _ => connection.exists(&keys),
    };
    match sent {
        Ok(count) => Ok(outcome::ok(Value::Int(count))),
        Err(error) => Ok(outcome::failed(method, error)),
    }
}
