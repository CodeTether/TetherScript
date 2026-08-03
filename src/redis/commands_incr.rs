//! `INCR` and `INCRBY`.
//!
//! These are the rate-limit counters. Both are atomic server-side, which is the
//! entire reason to use them: a read-modify-write from the client would race
//! across processes and let a burst slip through the limit. Both create the key at
//! zero when it is absent, so a limiter needs no separate initialisation step —
//! though it does still need an [`Connection::expire`] call to give the window an
//! end.

use super::connection::Connection;
use super::error::RedisError;

impl Connection {
    /// Atomically add one to the integer at `key`.
    ///
    /// # Arguments
    ///
    /// * `key` — Counter key. Created holding `0` before incrementing if absent.
    ///
    /// # Returns
    ///
    /// The value *after* the increment, so the first call on a fresh key returns
    /// `1`.
    ///
    /// # Errors
    ///
    /// [`RedisError::Server`] when the key holds a non-integer string
    /// (`ERR value is not an integer or out of range`) or the result would
    /// overflow 64 bits. Transport and protocol failures as elsewhere.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tetherscript::redis::{Config, Connection};
    ///
    /// # fn main() -> Result<(), tetherscript::redis::RedisError> {
    /// let mut connection = Connection::connect(&Config::default())?;
    /// let hits = connection.incr(b"rate:ip:203.0.113.7")?;
    /// if hits == 1 {
    ///     // First hit in this window: bound it so the counter cannot live forever.
    ///     connection.expire(b"rate:ip:203.0.113.7", 60)?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn incr(&mut self, key: &[u8]) -> Result<i64, RedisError> {
        self.command(&[&b"INCR"[..], key])?.integer("INCR")
    }

    /// Atomically add `delta` to the integer at `key`.
    ///
    /// # Arguments
    ///
    /// * `key` — Counter key, created at `0` when absent.
    /// * `delta` — Amount to add. Negative values decrement, which is why the
    ///   parameter is signed and no separate `DECRBY` helper exists.
    ///
    /// # Returns
    ///
    /// The value after the addition.
    ///
    /// # Errors
    ///
    /// As [`Connection::incr`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use tetherscript::redis::Connection;
    /// # fn run(connection: &mut Connection) -> Result<(), tetherscript::redis::RedisError> {
    /// let _remaining = connection.incrby(b"quota:tokens", -25)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn incrby(&mut self, key: &[u8], delta: i64) -> Result<i64, RedisError> {
        let amount = delta.to_string();
        self.command(&[&b"INCRBY"[..], key, amount.as_bytes()])?
            .integer("INCRBY")
    }
}
