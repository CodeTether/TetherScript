//! Socket setup for a Redis connection.
//!
//! Kept separate from the protocol code because address resolution and timeout
//! configuration are an operating-system concern, not a RESP one.

use std::net::{TcpStream, ToSocketAddrs};

use super::config::Config;
use super::error::RedisError;

/// Open a TCP connection and apply the configured timeouts.
///
/// # Arguments
///
/// * `config` — Address and timeout settings.
///
/// # Returns
///
/// A connected stream with read and write timeouts already set.
///
/// # Errors
///
/// [`RedisError::Transport`] when the host cannot be resolved, resolves to no
/// address, cannot be reached within `connect_timeout`, or rejects the timeout
/// settings. Every message names the host and port, because "connection refused"
/// without an address is unactionable.
pub(super) fn dial(config: &Config) -> Result<TcpStream, RedisError> {
    let target = (config.host.as_str(), config.port);
    let mut addresses = target.to_socket_addrs().map_err(|error| {
        RedisError::Transport(format!("resolve {}:{}: {error}", config.host, config.port))
    })?;
    let address = addresses.next().ok_or_else(|| {
        RedisError::Transport(format!(
            "resolve {}:{}: host resolved to no addresses",
            config.host, config.port
        ))
    })?;
    // connect_timeout, not plain connect: a black-holed address otherwise hangs
    // for the operating system's own multi-minute default.
    let stream = TcpStream::connect_timeout(&address, config.connect_timeout).map_err(|error| {
        RedisError::Transport(format!(
            "connect to {}:{}: {error}",
            config.host, config.port
        ))
    })?;
    stream
        .set_read_timeout(Some(config.read_timeout))
        .map_err(|error| RedisError::Transport(format!("set read timeout: {error}")))?;
    stream
        .set_write_timeout(Some(config.write_timeout))
        .map_err(|error| RedisError::Transport(format!("set write timeout: {error}")))?;
    Ok(stream)
}
