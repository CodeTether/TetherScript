//! Wrapping a host outcome in a tetherscript `Result`.
//!
//! One concern: the calling convention. Every `redis.*` method returns
//! [`Value::Result`], so a script uses `?` and `match` rather than being aborted by
//! a host panic — a cache miss or a `WRONGTYPE` is a recoverable condition, and
//! AGENTS.md reserves panics for bugs.
//!
//! # Two different failures, deliberately not merged
//!
//! * A **command** failure — the server said `WRONGTYPE`, or the socket died — is
//!   `Ok(Value::Result(Err(..)))`. The capability worked; the operation did not, so
//!   the script gets a value it can inspect.
//! * A **usage** failure — a misspelled method, a wrong argument type, a negative
//!   TTL — is a plain `Err(String)` from
//!   [`Authority::invoke`](crate::capability::Authority::invoke), the same as
//!   `fs.read` on a non-string path. That is a bug in the script, not a condition
//!   to be caught in a retry loop, so it surfaces as a language error.

use std::rc::Rc;

use crate::redis::RedisError;
use crate::value::{ResultValue, Value};

/// Wrap a successful command reply.
///
/// # Arguments
///
/// * `value` — The already-mapped reply.
///
/// # Returns
///
/// `Value::Result(Ok(value))`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis_cap::outcome;
/// use tetherscript::value::Value;
///
/// assert!(outcome::ok(Value::Int(1)).truthy());
/// ```
pub fn ok(value: Value) -> Value {
    Value::Result(Rc::new(ResultValue::Ok(value)))
}

/// Wrap a command failure as a catchable `Result`.
///
/// # Arguments
///
/// * `method` — Fully qualified method, so the message names the call site.
/// * `error` — The client's error.
///
/// # Returns
///
/// `Value::Result(Err(message))`, where the message is `<method>: <error>`.
/// [`RedisError`]'s own `Display` never contains a password: credentials live in
/// [`Config`](crate::redis::Config), which is deliberately not `Debug`, and the
/// handshake reports a failed `AUTH` by the server's reply text only.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis::RedisError;
/// use tetherscript::redis_cap::outcome;
///
/// let failed = outcome::failed("redis.get", RedisError::Transport("reset".into()));
/// assert!(!failed.truthy());
/// ```
pub fn failed(method: &str, error: RedisError) -> Value {
    Value::Result(Rc::new(ResultValue::Err(format!("{method}: {error}"))))
}
