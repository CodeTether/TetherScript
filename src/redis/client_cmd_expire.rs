//! `EXPIRE` and `TTL`: the expiry commands.
//!
//! `EXPIRE` answers `1` or `0`, and `0` is not an error: it means the key did not
//! exist, so nothing was given a deadline. That is returned as `bool` rather than
//! swallowed, because a rate limiter that ignores it silently creates a bucket
//! that never expires.
//!
//! `TTL` is mapped to [`Ttl`] rather than an integer; see `client_ttl.rs` for why.

use super::connection::Connection;
use super::error::ClientError;
use super::ttl::Ttl;

impl Connection {
    /// `EXPIRE key seconds`.
    ///
    /// # Arguments
    ///
    /// * `key` — Raw key bytes.
    /// * `seconds` — Time-to-live from now.
    ///
    /// # Returns
    ///
    /// `true` when the expiry was applied, `false` when the key does not exist.
    ///
    /// # Errors
    ///
    /// [`ClientError::Server`] when `seconds` is out of range,
    /// [`ClientError::UnexpectedType`] for a non-integer reply, or a transport
    /// error.
    pub fn expire(&mut self, key: &[u8], seconds: u64) -> Result<bool, ClientError> {
        let seconds = seconds.to_string();
        let applied = self
            .command(&[&b"EXPIRE"[..], key, seconds.as_bytes()])?
            .integer("EXPIRE")?;
        Ok(applied == 1)
    }

    /// `TTL key`, with the sentinels resolved.
    ///
    /// # Arguments
    ///
    /// * `key` — Raw key bytes.
    ///
    /// # Returns
    ///
    /// [`Ttl::Seconds`], [`Ttl::Persistent`] for `-1`, or [`Ttl::Missing`] for
    /// `-2`. A missing key is therefore never mistaken for a key about to expire.
    ///
    /// # Errors
    ///
    /// [`ClientError::UnexpectedType`] for a non-integer reply, or a transport
    /// error.
    pub fn ttl(&mut self, key: &[u8]) -> Result<Ttl, ClientError> {
        let reply = self.command(&[&b"TTL"[..], key])?.integer("TTL")?;
        Ok(Ttl::from_reply(reply))
    }
}
