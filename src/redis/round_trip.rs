//! The request/response round trip.
//!
//! Redis is request/response per connection, so a command writes its whole
//! encoded frame, then reads until the decoder reports a complete reply. The read
//! loop is where incremental decoding earns its keep: a reply larger than the
//! socket buffer arrives in pieces, and each partial read simply appends and
//! retries rather than failing.

use std::io::{Read, Write};

use super::connection::Connection;
use super::decode::{Decoded, decode};
use super::error::RedisError;
use super::value::RespValue;

/// One read of the socket. Small because most replies are tiny; large replies
/// just loop.
const CHUNK: usize = 8 * 1024;

impl Connection {
    /// Send an already-encoded command and read exactly one reply.
    ///
    /// # Arguments
    ///
    /// * `request` — Bytes from `encode_command`.
    ///
    /// # Returns
    ///
    /// The decoded reply, including [`RespValue::Error`] when the server reported
    /// a command failure; promotion to [`RedisError::Server`] happens one layer up
    /// in the `request` module.
    ///
    /// # Errors
    ///
    /// * [`RedisError::Transport`] on a write failure, a read failure or timeout,
    ///   or EOF while a reply was still owed.
    /// * [`RedisError::Protocol`] when the bytes are not valid RESP.
    pub(super) fn round_trip(&mut self, request: &[u8]) -> Result<RespValue, RedisError> {
        self.send(request)?;
        self.read_reply()
    }

    /// Write a whole request and flush it.
    fn send(&mut self, request: &[u8]) -> Result<(), RedisError> {
        self.stream
            .write_all(request)
            .map_err(|error| RedisError::Transport(format!("write command: {error}")))?;
        self.stream
            .flush()
            .map_err(|error| RedisError::Transport(format!("flush command: {error}")))
    }

    /// Read until one complete reply is decoded, draining only its bytes.
    fn read_reply(&mut self) -> Result<RespValue, RedisError> {
        loop {
            if let Decoded::Frame { value, consumed } = decode(&self.buffer)? {
                self.buffer.drain(..consumed);
                return Ok(value);
            }
            let mut chunk = [0u8; CHUNK];
            match self.stream.read(&mut chunk) {
                // A clean EOF mid-reply is a transport failure, not an empty
                // reply: the server closed while it still owed us bytes.
                Ok(0) => {
                    return Err(RedisError::Transport(
                        "server closed the connection with a reply outstanding".into(),
                    ));
                }
                Ok(read) => self.buffer.extend_from_slice(&chunk[..read]),
                Err(error) => {
                    return Err(RedisError::Transport(format!("read reply: {error}")));
                }
            }
        }
    }
}
