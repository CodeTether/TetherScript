//! Dialling a real server: resolve, connect with a deadline, set timeouts.
//!
//! Split from [`Connection`] because socket setup is an operating-system concern.

use std::net::{TcpStream, ToSocketAddrs};

use super::codec::RespCodec;
use super::config::Config;
use super::connection::Connection;
use super::error::ClientError;

impl Connection {
    /// Connect, optionally `AUTH`, and optionally `SELECT` a database.
    ///
    /// # Arguments
    ///
    /// * `config` — Address, credentials, database index, and timeouts.
    /// * `codec` — The RESP boundary implementation.
    ///
    /// # Returns
    ///
    /// A connection whose handshake has completed.
    ///
    /// # Errors
    ///
    /// [`ClientError::Transport`] when the host cannot be resolved or reached
    /// within `connect_timeout`, or the timeout options are rejected;
    /// [`ClientError::Server`] when the password is wrong or the database index is
    /// out of range, since those are error *replies* and the transport worked.
    ///
    /// # Examples
    ///
    /// Requires a live server; see the [`Connection`] example, marked `no_run`.
    pub fn connect(config: &Config, codec: Box<dyn RespCodec>) -> Result<Self, ClientError> {
        let stream = dial(config)?;
        let mut connection = Self::from_parts(Box::new(stream), codec);
        connection.handshake(config)?;
        Ok(connection)
    }
}

/// Resolve `config`'s address and open a timeout-bounded TCP stream.
///
/// # Errors
///
/// [`ClientError::Transport`], always naming host and port: "connection refused"
/// without an address is unactionable.
fn dial(config: &Config) -> Result<TcpStream, ClientError> {
    let target = format!("{}:{}", config.host, config.port);
    let fail = |detail: String| ClientError::Transport(format!("{target}: {detail}"));
    let address = target
        .to_socket_addrs()
        .map_err(|error| fail(format!("resolve: {error}")))?
        .next()
        .ok_or_else(|| fail("resolve: host resolved to no addresses".into()))?;
    // connect_timeout, not plain connect: a black-holed address otherwise hangs
    // for the operating system's own multi-minute default.
    let stream = TcpStream::connect_timeout(&address, config.connect_timeout)
        .map_err(|error| fail(format!("connect: {error}")))?;
    stream
        .set_read_timeout(Some(config.read_timeout))
        .and_then(|()| stream.set_write_timeout(Some(config.write_timeout)))
        .map_err(|error| fail(format!("set timeouts: {error}")))?;
    Ok(stream)
}
