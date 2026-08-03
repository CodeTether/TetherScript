//! Counter operations: `INCR` and `DECR`.
//!
//! Separated from plain string reads and writes because these are the commands
//! whose *server-side* failure mode matters for pooling: `INCR` on a key holding
//! `"abc"` answers `ERR value is not an integer or out of range`. That reply is
//! fully drained, so the connection is released and remains usable — see
//! [`super::handler_exec`].

use super::handler::RedisHandler;
use super::handler_command::arg;
use crate::value::Value;

impl RedisHandler {
    /// Increment a key by one, treating a missing key as zero.
    ///
    /// # Arguments
    ///
    /// * `key` — Key name.
    ///
    /// # Returns
    ///
    /// [`Value::Int`] holding the value after the increment.
    ///
    /// # Errors
    ///
    /// Returns the server's error when the key holds a non-numeric string or the
    /// result would overflow a 64-bit integer, plus transport and pool errors. A
    /// non-numeric error does not disturb the connection: the next command on it
    /// succeeds normally.
    pub fn incr(&self, key: &str) -> Result<Value, String> {
        self.command(&[arg("INCR"), arg(key)])
    }

    /// Decrement a key by one, treating a missing key as zero.
    ///
    /// # Arguments
    ///
    /// * `key` — Key name.
    ///
    /// # Returns
    ///
    /// [`Value::Int`] holding the value after the decrement.
    ///
    /// # Errors
    ///
    /// Same conditions as [`RedisHandler::incr`].
    pub fn decr(&self, key: &str) -> Result<Value, String> {
        self.command(&[arg("DECR"), arg(key)])
    }
}
