//! The three `SET` shapes: `set`, `setex`, `setnx`.
//!
//! One concern: writing a whole string value. They share a body because they differ
//! only in their [`SetOptions`], and the order of the modifiers on the wire is the
//! encoder's business, not this module's.

use crate::redis::{Connection, SetOptions};
use crate::redis_cap::{args, coerce_bytes, coerce_int, coerce_seconds, outcome};
use crate::value::Value;

/// `redis.set(key, value)` — store a value with no expiry.
///
/// # Returns
///
/// `Ok(Result::Ok(true))`. Note that a plain `SET` also **clears** any existing TTL
/// on the key, per Redis semantics; use [`setex`] to store a value with a lifetime.
///
/// # Errors
///
/// A usage `Err` for a wrong arity or a non-string `key` or `value`; a catchable
/// `Result::Err` on server or transport failure.
pub(super) fn set(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    args::exactly("redis.set", arguments, 2)?;
    write(connection, "redis.set", arguments, SetOptions::default())
}

/// `redis.setex(key, value, seconds)` — store a value and its lifetime atomically.
///
/// Atomic on purpose: `SET` followed by `EXPIRE` leaves a window where a crash in
/// between leaves a key that never expires, which is how a session store fills up.
///
/// # Returns
///
/// `Ok(Result::Ok(true))`.
///
/// # Errors
///
/// A usage `Err` naming `seconds` when it is not an int, or when it is zero or
/// negative — see [`coerce_seconds`], where a non-positive TTL would delete the key.
pub(super) fn setex(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    args::exactly("redis.setex", arguments, 3)?;
    let raw = args::at("redis.setex", "seconds", arguments, 2)?;
    let seconds = coerce_int::int("redis.setex", "seconds", raw)?;
    let checked = coerce_seconds::positive("redis.setex", "seconds", seconds)?;
    write(
        connection,
        "redis.setex",
        arguments,
        SetOptions::expiring(checked),
    )
}

/// `redis.setnx(key, value)` — store only if the key is absent.
///
/// # Returns
///
/// `Ok(Result::Ok(true))` when this call created the key, `Ok(Result::Ok(false))`
/// when it already existed. That distinction is the lock and rate-limit primitive:
/// an `exists` followed by a `set` cannot report it without a race.
///
/// # Errors
///
/// As [`set`].
pub(super) fn setnx(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    args::exactly("redis.setnx", arguments, 2)?;
    let options = SetOptions {
        expire_seconds: None,
        if_not_exists: true,
    };
    write(connection, "redis.setnx", arguments, options)
}

/// Shared body: coerce `key` and `value`, send, and map the reply.
///
/// # Errors
///
/// A usage `Err` naming `key` or `value` on a type mismatch.
fn write(
    connection: &mut Connection,
    method: &str,
    arguments: &[Value],
    options: SetOptions,
) -> Result<Value, String> {
    let key = coerce_bytes::bytes(method, "key", args::at(method, "key", arguments, 0)?)?;
    let value = coerce_bytes::bytes(method, "value", args::at(method, "value", arguments, 1)?)?;
    match connection.set(&key, &value, &options) {
        Ok(stored) => Ok(outcome::ok(Value::Bool(stored))),
        Err(error) => Ok(outcome::failed(method, error)),
    }
}
