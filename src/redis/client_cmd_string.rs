//! `GET`, `SET`, and `SET … EX`: the string commands.
//!
//! `get` returns `Option<Vec<u8>>` rather than `Vec<u8>`: `None` is a missing key
//! and `Some(vec![])` is a key holding the empty string. Conflating them turns a
//! cache miss into a cached empty value, which then never recomputes.
//!
//! `set_ex` exists separately from `set` because `SET k v EX n` is atomic, while
//! `SET` followed by `EXPIRE` leaves a window in which a crash leaks a key that
//! never expires — which is how session stores fill up.

use super::connection::Connection;
use super::error::ClientError;

impl Connection {
    /// `GET key`.
    ///
    /// # Arguments
    ///
    /// * `key` — Raw key bytes.
    ///
    /// # Returns
    ///
    /// `Some(bytes)` when the key exists, including `Some(vec![])` for an empty
    /// value; `None` only when the key is absent.
    ///
    /// # Errors
    ///
    /// [`ClientError::Server`] for a wrong-type key, [`ClientError::UnexpectedType`]
    /// for a non-string reply, or a transport error.
    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, ClientError> {
        self.command(&[&b"GET"[..], key])?.optional_bulk("GET")
    }

    /// `SET key value`, overwriting unconditionally and clearing any expiry.
    ///
    /// # Arguments
    ///
    /// * `key` — Raw key bytes.
    /// * `value` — Raw value bytes; may be empty.
    ///
    /// # Errors
    ///
    /// [`ClientError::Server`], [`ClientError::UnexpectedType`], or a transport
    /// error.
    pub fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), ClientError> {
        self.command(&[&b"SET"[..], key, value]).map(|_| ())
    }

    /// `SET key value EX seconds`, setting the value and its expiry atomically.
    ///
    /// # Arguments
    ///
    /// * `key` — Raw key bytes.
    /// * `value` — Raw value bytes.
    /// * `seconds` — Time-to-live. Redis rejects `0`, so the caller must not pass it.
    ///
    /// # Errors
    ///
    /// [`ClientError::Server`] when `seconds` is out of range, plus the usual
    /// type and transport errors.
    pub fn set_ex(&mut self, key: &[u8], value: &[u8], seconds: u64) -> Result<(), ClientError> {
        let seconds = seconds.to_string();
        self.command(&[&b"SET"[..], key, value, &b"EX"[..], seconds.as_bytes()])
            .map(|_| ())
    }
}
