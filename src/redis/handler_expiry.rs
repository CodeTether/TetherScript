//! Key lifetime operations: `EXPIRE` and `TTL`.
//!
//! One concern: how long a key lives. Kept apart from reads and writes because the
//! two TTL sentinel values (`-1` and `-2`) are a distinct piece of semantics a
//! caller has to understand.

use super::handler::RedisHandler;
use super::handler_command::arg;
use crate::value::Value;

impl RedisHandler {
    /// Set a key's time-to-live in seconds.
    ///
    /// # Arguments
    ///
    /// * `key` — Key name.
    /// * `seconds` — Lifetime from now. Redis deletes the key immediately when this
    ///   is zero or negative.
    ///
    /// # Returns
    ///
    /// [`Value::Int`] holding `1` when the timeout was set, or `0` when the key does
    /// not exist. A missing key is *not* an error, so a caller that needs to know
    /// must check the `0`.
    ///
    /// # Errors
    ///
    /// Returns transport, pool, or server errors.
    pub fn expire(&self, key: &str, seconds: i64) -> Result<Value, String> {
        self.command(&[arg("EXPIRE"), arg(key), arg(&seconds.to_string())])
    }

    /// Read a key's remaining time-to-live in seconds.
    ///
    /// # Arguments
    ///
    /// * `key` — Key name.
    ///
    /// # Returns
    ///
    /// [`Value::Int`]: a positive count of seconds remaining, `-1` when the key
    /// exists with no expiry, or `-2` when the key does not exist. The two negative
    /// sentinels are passed through unchanged rather than collapsed to `Nil`,
    /// because "no expiry" and "no key" call for different handling.
    ///
    /// # Errors
    ///
    /// Returns transport, pool, or server errors.
    pub fn ttl(&self, key: &str) -> Result<Value, String> {
        self.command(&[arg("TTL"), arg(key)])
    }
}
