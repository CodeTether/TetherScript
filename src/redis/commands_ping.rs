//! `PING`.
//!
//! The liveness check, and the one command a pool should use to decide whether a
//! reused connection is still good.

use super::connection::Connection;
use super::error::RedisError;

impl Connection {
    /// Send `PING` and read the status reply.
    ///
    /// # Returns
    ///
    /// The status text, `"PONG"` on a healthy server.
    ///
    /// # Errors
    ///
    /// * [`RedisError::Transport`] when the socket is dead or the read times out.
    ///   This is the signal a pool wants: the connection must be discarded.
    /// * [`RedisError::Server`] when the server refuses, for instance `NOAUTH` on
    ///   a password-protected server before `AUTH`.
    /// * [`RedisError::UnexpectedType`] if the reply is not a status line.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tetherscript::redis::{Config, Connection};
    ///
    /// # fn main() -> Result<(), tetherscript::redis::RedisError> {
    /// let mut connection = Connection::connect(&Config::default())?;
    /// assert_eq!(connection.ping()?, "PONG");
    /// # Ok(())
    /// # }
    /// ```
    pub fn ping(&mut self) -> Result<String, RedisError> {
        Ok(self.command(&[&b"PING"[..]])?.simple("PING")?.to_string())
    }
}
