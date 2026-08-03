//! `redis.get` — the read path.
//!
//! One concern: reading a key, and preserving the difference between *absent* and
//! *empty* while doing it. Kept separate from [`super::methods_set`] because the
//! nil-vs-empty distinction is the subtle part of the capability and deserves to be
//! read on its own.

use crate::redis::Connection;
use crate::redis_cap::{args, coerce_bytes, outcome, reply};
use crate::value::Value;

/// `redis.get(key)` — read a key.
///
/// # Arguments
///
/// * `connection` — Borrowed for exactly this one command.
/// * `arguments` — Exactly one: `key`, a `str` or `bytes`.
///
/// # Returns
///
/// `Ok(Result::Ok(value))`, where the value is `nil` for a **missing** key and `""`
/// for a key holding the empty string. Those are different answers — a cache miss
/// versus a cached empty value — and [`reply`] documents why they are never merged.
/// A value that is not valid UTF-8 comes back as `bytes`.
///
/// # Errors
///
/// A usage `Err(String)` for a wrong arity or a non-string key. A server or
/// transport failure such as `WRONGTYPE` is a catchable `Result::Err` value instead.
pub(super) fn get(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    args::exactly("redis.get", arguments, 1)?;
    let key = coerce_bytes::bytes(
        "redis.get",
        "key",
        args::at("redis.get", "key", arguments, 0)?,
    )?;
    match connection.get(&key) {
        Ok(payload) => Ok(outcome::ok(reply::optional_bulk(payload))),
        Err(error) => Ok(outcome::failed("redis.get", error)),
    }
}
