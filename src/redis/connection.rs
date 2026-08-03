//! A single Redis connection over TCP.
//!
//! Deliberately synchronous and blocking, matching
//! [`crate::postgres::Connection`]: the runtime is single-threaded, so a
//! request/response round trip is the honest shape of the operation. Timeouts,
//! not async, are what keep a dead server from wedging the process.
//!
//! Redis is strictly request/response per connection, so the connection owns a
//! read buffer that may hold bytes belonging to a *later* reply — a server can
//! coalesce writes, so one `read` can deliver more than one frame. The buffer is
//! therefore drained by exactly the number of bytes the decoder consumed, never
//! cleared wholesale.

use std::net::TcpStream;

use super::config::Config;
use super::error::RedisError;

/// An authenticated connection ready to accept commands.
///
/// Created by [`Connection::connect`], which finishes optional `AUTH` and
/// `SELECT` before returning, so a value of this type is always ready to use.
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
pub struct Connection {
    pub(super) stream: TcpStream,
    /// Bytes read but not yet decoded, including any pipelined remainder.
    pub(super) buffer: Vec<u8>,
}

/// Deliberately opaque: the settings that reached this connection include a
/// password, and a panic message must never print one.
impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Connection(..)")
    }
}

impl Connection {
    /// Connect, optionally `AUTH`, and optionally `SELECT` a database.
    ///
    /// # Arguments
    ///
    /// * `config` — Address, credentials, database index, and timeouts.
    ///
    /// # Returns
    ///
    /// A connection whose handshake has completed.
    ///
    /// # Errors
    ///
    /// * [`RedisError::Transport`] when resolution, connect, or the timeout
    ///   settings fail.
    /// * [`RedisError::Server`] when the password is wrong (`WRONGPASS`), no
    ///   password is configured server-side (`ERR Client sent AUTH ...`), or the
    ///   database index is out of range. These are error *replies*: the transport
    ///   worked.
    ///
    /// # Examples
    ///
    /// See the [`Connection`] example; connecting needs a live server, so it is
    /// marked `no_run`.
    pub fn connect(config: &Config) -> Result<Self, RedisError> {
        let stream = super::connection_socket::dial(config)?;
        let mut connection = Self {
            stream,
            buffer: Vec::new(),
        };
        super::handshake::run(&mut connection, config)?;
        Ok(connection)
    }
}
