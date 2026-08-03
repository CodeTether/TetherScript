//! Coercing a script argument into an integer.
//!
//! One concern: `int`-typed parameters (`delta`, and the raw side of the second/TTL
//! parameters). Range validation for durations lives in [`super::coerce_seconds`],
//! so this module never decides what a *sensible* number is — only what is a
//! number at all.

use super::args_error;
use crate::value::Value;

/// Coerce an argument into an `i64`.
///
/// # Arguments
///
/// * `method` — Fully qualified method, for the error message.
/// * `parameter` — Parameter name, for the error message.
/// * `value` — The supplied argument.
///
/// # Returns
///
/// The integer, unchanged. Negative values are returned as-is: `redis.incrby` is
/// legitimately how a script decrements.
///
/// # Errors
///
/// Returns [`args_error::mismatch`] naming `parameter` and the actual type.
/// A `float` is refused rather than truncated, because `incrby(k, 1.9)` adding 1 is
/// a silent wrong answer, and a `str` is refused rather than parsed, so a typo in a
/// numeric literal surfaces at the call rather than at the server.
///
/// # Examples
///
/// ```rust
/// use std::rc::Rc;
/// use tetherscript::redis_cap::coerce_int;
/// use tetherscript::value::Value;
///
/// assert_eq!(coerce_int::int("redis.incrby", "delta", &Value::Int(-5)).unwrap(), -5);
///
/// let error = coerce_int::int("redis.incrby", "delta", &Value::Float(1.9)).unwrap_err();
/// assert_eq!(error, "redis.incrby: parameter `delta` must be an int, got float");
///
/// let text = Value::Str(Rc::new("5".to_string()));
/// assert!(coerce_int::int("redis.incrby", "delta", &text).is_err());
/// ```
pub fn int(method: &str, parameter: &str, value: &Value) -> Result<i64, String> {
    match value {
        Value::Int(number) => Ok(*number),
        other => Err(args_error::mismatch(method, parameter, "an int", other)),
    }
}
