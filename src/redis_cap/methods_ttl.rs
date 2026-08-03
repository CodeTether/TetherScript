//! Key lifetime methods: `expire` and `ttl`.
//!
//! One concern: how long a key lives. The validation that makes `expire` safe lives
//! in [`coerce_seconds`] and the three-way `TTL` mapping in
//! [`reply_ttl`](crate::redis_cap::reply_ttl); this module only wires them to the
//! connection.

use crate::redis::Connection;
use crate::redis_cap::{args, coerce_bytes, coerce_int, coerce_seconds, outcome, reply_ttl};
use crate::value::Value;

/// `redis.expire(key, seconds)` — set a lifetime on an existing key.
///
/// # Arguments
///
/// * `arguments` — `key`, then `seconds`, a **positive** int.
///
/// # Returns
///
/// `Ok(Result::Ok(true))` when the timeout was applied, `Ok(Result::Ok(false))` when
/// the key does not exist. `false` is not an error: expiring an absent key is a
/// no-op, and reporting it as a value lets a caller decide whether that matters.
///
/// # Errors
///
/// A usage `Err` naming `seconds` when it is not an int, or when it is zero or
/// negative. Redis reads `EXPIRE key 0` as *delete now*, so forwarding a computed
/// `0` would silently destroy a rate-limit counter or log a user out; use
/// `redis.del` when deletion is what you mean.
pub(super) fn expire(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    args::exactly("redis.expire", arguments, 2)?;
    let key_arg = args::at("redis.expire", "key", arguments, 0)?;
    let key = coerce_bytes::bytes("redis.expire", "key", key_arg)?;
    let raw = args::at("redis.expire", "seconds", arguments, 1)?;
    let seconds = coerce_int::int("redis.expire", "seconds", raw)?;
    let checked = coerce_seconds::positive("redis.expire", "seconds", seconds)?;
    match connection.expire(&key, checked) {
        Ok(applied) => Ok(outcome::ok(Value::Bool(applied))),
        Err(error) => Ok(outcome::failed("redis.expire", error)),
    }
}

/// `redis.ttl(key)` — read a key's remaining lifetime.
///
/// # Arguments
///
/// * `arguments` — Exactly one: `key`.
///
/// # Returns
///
/// `Ok(Result::Ok(v))` where `v` is an int of seconds remaining, `false` for a key
/// that exists but never expires, or `nil` for a key that does not exist. The raw
/// `-1`/`-2` sentinels are never handed to a script; see
/// [`reply_ttl`](crate::redis_cap::reply_ttl) for why.
///
/// # Errors
///
/// A usage `Err` for a wrong arity or a non-string key; a catchable `Result::Err` on
/// transport failure.
pub(super) fn ttl(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    args::exactly("redis.ttl", arguments, 1)?;
    let raw = args::at("redis.ttl", "key", arguments, 0)?;
    let key = coerce_bytes::bytes("redis.ttl", "key", raw)?;
    match connection.ttl(&key) {
        Ok(remaining) => Ok(outcome::ok(reply_ttl::value(remaining))),
        Err(error) => Ok(outcome::failed("redis.ttl", error)),
    }
}
