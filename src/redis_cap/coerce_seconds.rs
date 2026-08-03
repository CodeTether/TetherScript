//! Validating a duration argument: `setex(seconds)` and `expire(seconds)`.
//!
//! One concern: refusing a lifetime Redis would misinterpret. This is separate from
//! [`super::coerce_int`] because being *an integer* and being *a usable duration*
//! are different questions, and only the second one is security-relevant.
//!
//! # Why a non-positive TTL is refused rather than forwarded
//!
//! Redis treats these three inputs very differently, and the difference is
//! destructive:
//!
//! * `EXPIRE key 0` and `EXPIRE key -1` **delete the key immediately**. A rate
//!   limiter that computed a window of `0` from a clock skew would silently clear
//!   its counter instead of throttling, and a session store would log the user out.
//! * `SETEX key 0 value` is rejected by the server with
//!   `ERR invalid expire time`, so the value is never stored — but the script only
//!   learns this from a server round trip.
//!
//! Neither outcome is what a caller writing `expire(key, n)` means when `n` came out
//! as `0`. So the capability refuses locally, naming the parameter and the value,
//! before any byte reaches the socket. Deletion is available deliberately, through
//! `redis.del`.
//!
//! The value *is* printed here, unlike elsewhere in this capability: a duration is
//! not a secret, and the number is the whole point of the diagnostic.

/// Validate a positive duration in seconds.
///
/// # Arguments
///
/// * `method` — Fully qualified method, for the error message.
/// * `parameter` — Parameter name, for the error message.
/// * `seconds` — The already-coerced integer.
///
/// # Returns
///
/// The value as `u64`, which is what
/// [`Connection::expire`](crate::redis::Connection::expire) and
/// [`SetOptions::expire_seconds`](crate::redis::SetOptions) both take.
///
/// # Errors
///
/// Returns an error naming `parameter` and the offending value for zero and for any
/// negative number.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis_cap::coerce_seconds;
///
/// assert_eq!(coerce_seconds::positive("redis.expire", "seconds", 60).unwrap(), 60);
///
/// // Zero would delete the key.
/// assert!(coerce_seconds::positive("redis.expire", "seconds", 0).is_err());
///
/// // A negative TTL names the parameter.
/// let error = coerce_seconds::positive("redis.setex", "seconds", -1).unwrap_err();
/// assert!(error.contains("`seconds`"), "got: {error}");
/// ```
pub fn positive(method: &str, parameter: &str, seconds: i64) -> Result<u64, String> {
    if seconds <= 0 {
        return Err(format!(
            "{method}: parameter `{parameter}` must be a positive number of seconds, \
             got {seconds}; use redis.del to remove a key"
        ));
    }
    Ok(seconds as u64)
}
