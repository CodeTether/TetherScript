//! `EXPIRE` and `TTL`.
//!
//! `TTL`'s two negative sentinels are the subtle part and are surfaced as a real
//! type, [`Ttl`], rather than a raw `i64` that every caller must remember to
//! interpret:
//!
//! | Reply | Meaning |
//! |---|---|
//! | `>= 0` | Seconds remaining |
//! | `-1` | The key exists but has no expiry |
//! | `-2` | The key does not exist |
//!
//! Treating `-1` and `-2` as "expired" is a common bug: a persistent session and a
//! deleted one are different states, and a cache that conflates them re-renders
//! entries that were meant to be permanent.

use super::connection::Connection;
use super::error::RedisError;
use super::ttl::Ttl;

impl Connection {
    /// Set a key's time-to-live in seconds.
    ///
    /// # Arguments
    ///
    /// * `key` — Key to bound.
    /// * `seconds` — Lifetime from now.
    ///
    /// # Returns
    ///
    /// `true` when the timeout was set, `false` when the key does not exist. Redis
    /// answers `:0` in the latter case, so a missing key is not an error.
    ///
    /// # Errors
    ///
    /// [`RedisError::Server`], [`RedisError::Transport`], or
    /// [`RedisError::Protocol`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use tetherscript::redis::Connection;
    /// # fn run(connection: &mut Connection) -> Result<(), tetherscript::redis::RedisError> {
    /// let _bounded = connection.expire(b"rate:ip:203.0.113.7", 60)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn expire(&mut self, key: &[u8], seconds: u64) -> Result<bool, RedisError> {
        let ttl = seconds.to_string();
        let applied = self
            .command(&[&b"EXPIRE"[..], key, ttl.as_bytes()])?
            .integer("EXPIRE")?;
        Ok(applied == 1)
    }

    /// Read a key's remaining time-to-live.
    ///
    /// # Arguments
    ///
    /// * `key` — Key to inspect.
    ///
    /// # Returns
    ///
    /// A [`Ttl`], which names all three outcomes rather than returning a signed
    /// count the caller must decode.
    ///
    /// # Errors
    ///
    /// [`RedisError::Server`], [`RedisError::Transport`], or
    /// [`RedisError::Protocol`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tetherscript::redis::Ttl;
    /// # use tetherscript::redis::Connection;
    /// # fn run(connection: &mut Connection) -> Result<(), tetherscript::redis::RedisError> {
    /// match connection.ttl(b"session:42")? {
    ///     Ttl::Seconds(left) => println!("{left}s left"),
    ///     Ttl::Persistent => println!("never expires"),
    ///     Ttl::Missing => println!("gone"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn ttl(&mut self, key: &[u8]) -> Result<Ttl, RedisError> {
        let reply = self.command(&[&b"TTL"[..], key])?.integer("TTL")?;
        Ok(Ttl::from_reply(reply))
    }
}
