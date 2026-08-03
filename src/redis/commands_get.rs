//! `GET`.
//!
//! Returns bytes, not a `String`, because Redis values are binary-safe: a render
//! cache storing a PNG or a gzip frame must round-trip unchanged.

use super::connection::Connection;
use super::error::RedisError;

impl Connection {
    /// Read the value at `key`.
    ///
    /// # Arguments
    ///
    /// * `key` — Key bytes. Binary keys are fine; nothing is parsed.
    ///
    /// # Returns
    ///
    /// `Some(bytes)` when the key exists, `None` when it does not. A key holding
    /// the empty string returns `Some(vec![])`, which is a different answer from
    /// `None`: the server sent `$0\r\n\r\n` rather than `$-1\r\n`. A session store
    /// must not collapse the two.
    ///
    /// # Errors
    ///
    /// * [`RedisError::Server`] when the key holds a non-string type
    ///   (`WRONGTYPE`).
    /// * [`RedisError::Transport`] or [`RedisError::Protocol`] on socket or framing
    ///   failure.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tetherscript::redis::{Config, Connection};
    ///
    /// # fn main() -> Result<(), tetherscript::redis::RedisError> {
    /// let mut connection = Connection::connect(&Config::default())?;
    /// match connection.get(b"session:42")? {
    ///     Some(bytes) => println!("{} bytes", bytes.len()),
    ///     None => println!("no such session"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, RedisError> {
        let reply = self.command(&[&b"GET"[..], key])?;
        Ok(reply.optional_bulk("GET")?.map(<[u8]>::to_vec))
    }
}
