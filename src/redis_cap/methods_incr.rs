//! Counter methods: `incr` and `incrby`.
//!
//! One concern: atomic integer arithmetic on a key. This is the rate-limit
//! primitive, and the increment happens *server-side* on purpose: a
//! read-modify-write from the script would lose counts under concurrency, which is
//! exactly the case a rate limiter exists to handle.
//!
//! There is deliberately no `decr`/`decrby`. A negative `delta` to [`incrby`] is the
//! same operation, and one code path cannot disagree with itself about overflow.

use crate::redis::{Connection, RedisError};
use crate::redis_cap::{args, coerce_bytes, coerce_int, outcome};
use crate::value::Value;

/// `redis.incr(key)` — add one to the integer at `key`.
///
/// # Arguments
///
/// * `arguments` — Exactly one: `key`, a `str` or `bytes`.
///
/// # Returns
///
/// `Ok(Result::Ok(n))`, the value **after** the increment. A missing key is created
/// at `0` first, so the first call returns `1`.
///
/// # Errors
///
/// A usage `Err` for a wrong arity or a non-string key. A key holding a non-integer
/// string is a catchable `Result::Err` carrying the server's
/// `ERR value is not an integer or out of range`.
pub(super) fn incr(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    args::exactly("redis.incr", arguments, 1)?;
    let raw = args::at("redis.incr", "key", arguments, 0)?;
    let key = coerce_bytes::bytes("redis.incr", "key", raw)?;
    map(connection.incr(&key), "redis.incr")
}

/// `redis.incrby(key, delta)` — add `delta` to the integer at `key`.
///
/// # Arguments
///
/// * `arguments` — `key` then `delta`, an int.
///
/// # Returns
///
/// `Ok(Result::Ok(n))`, the value after the addition. `delta` may be negative, which
/// is how a script decrements.
///
/// # Errors
///
/// A usage `Err` naming `delta` when it is not an int; a `float` is refused rather
/// than truncated, since `incrby(k, 1.9)` adding `1` is a silent wrong answer.
pub(super) fn incrby(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    args::exactly("redis.incrby", arguments, 2)?;
    let key_arg = args::at("redis.incrby", "key", arguments, 0)?;
    let key = coerce_bytes::bytes("redis.incrby", "key", key_arg)?;
    let delta_arg = args::at("redis.incrby", "delta", arguments, 1)?;
    let delta = coerce_int::int("redis.incrby", "delta", delta_arg)?;
    map(connection.incrby(&key, delta), "redis.incrby")
}

/// Wrap a counter reply, which is always an integer on success.
///
/// # Errors
///
/// Never returns a usage `Err`; a client failure becomes a catchable
/// `Result::Err` value.
fn map(sent: Result<i64, RedisError>, method: &str) -> Result<Value, String> {
    match sent {
        Ok(count) => Ok(outcome::ok(Value::Int(count))),
        Err(error) => Ok(outcome::failed(method, error)),
    }
}
