//! `INCR` and `INCRBY`: the counter commands the rate limiter needs.
//!
//! Both create the key at zero before incrementing, which is what makes
//! `INCR` then `EXPIRE` a correct fixed-window bucket: the first request in a
//! window is the one that gets `1` back, so it is unambiguously the one that
//! should set the deadline.
//!
//! A non-numeric existing value is a server error reply (`ERR value is not an
//! integer or out of range`), not a transport fault, so the connection survives it.

use super::connection::Connection;
use super::error::ClientError;

impl Connection {
    /// `INCR key`.
    ///
    /// # Arguments
    ///
    /// * `key` — Raw key bytes. Created at `0` first when absent.
    ///
    /// # Returns
    ///
    /// The value after incrementing; `1` for a key that did not exist.
    ///
    /// # Errors
    ///
    /// [`ClientError::Server`] when the value is not an integer or would overflow,
    /// [`ClientError::UnexpectedType`] for a non-integer reply, or a transport
    /// error.
    pub fn incr(&mut self, key: &[u8]) -> Result<i64, ClientError> {
        self.command(&[&b"INCR"[..], key])?.integer("INCR")
    }

    /// `INCRBY key delta`.
    ///
    /// # Arguments
    ///
    /// * `key` — Raw key bytes.
    /// * `delta` — Amount to add; negative values decrement.
    ///
    /// # Returns
    ///
    /// The value after adding `delta`.
    ///
    /// # Errors
    ///
    /// As [`Connection::incr`].
    pub fn incr_by(&mut self, key: &[u8], delta: i64) -> Result<i64, ClientError> {
        let delta = delta.to_string();
        self.command(&[&b"INCRBY"[..], key, delta.as_bytes()])?
            .integer("INCRBY")
    }
}
