//! A single Redis connection.
//!
//! Deliberately synchronous and blocking, matching
//! [`crate::postgres::Connection`]: a request/response round trip is the honest
//! shape of the operation, and timeouts rather than async are what stop a dead
//! server from wedging the process.
//!
//! Redis is strictly request/response per connection, so the connection owns the
//! read buffer. One `read` can deliver more than one reply because a server may
//! coalesce writes, so the buffer is drained by exactly the byte count the codec
//! consumed and is never cleared wholesale.

use super::codec::RespCodec;
use super::transport::Transport;

/// A connection whose handshake has completed and which is ready for commands.
///
/// Build one with [`Connection::connect`] for real use, or
/// [`Connection::from_parts`] to drive a scripted transport in tests.
///
/// # Examples
///
/// ```rust,ignore
/// use tetherscript::redis::client::{Config, Connection};
///
/// let mut connection = Connection::connect(&Config::default(), Box::new(RespAdapter))?;
/// let cached = connection.get(b"render:home")?;
/// ```
pub struct Connection {
    pub(super) transport: Box<dyn Transport>,
    pub(super) codec: Box<dyn RespCodec>,
    /// Bytes read but not yet framed, including any pipelined remainder.
    pub(super) buffer: Vec<u8>,
}

/// Deliberately opaque. The settings that produced this connection carried a
/// password, so a derived `Debug` here would risk printing it transitively; see
/// `client_config_debug.rs`.
impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Connection(..)")
    }
}

impl Connection {
    /// Wrap an already-open transport and a codec, running no handshake.
    ///
    /// # Arguments
    ///
    /// * `transport` — An open byte stream.
    /// * `codec` — The RESP boundary implementation.
    ///
    /// # Returns
    ///
    /// A connection that will issue its next command on `transport`.
    pub fn from_parts(transport: Box<dyn Transport>, codec: Box<dyn RespCodec>) -> Self {
        Self {
            transport,
            codec,
            buffer: Vec::new(),
        }
    }
}
