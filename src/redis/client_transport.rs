//! # The byte-stream boundary, and its TCP implementation
//!
//! The exchange loop needs only two operations from the network: push all of a
//! request out, and pull some bytes back. Naming that as [`Transport`] rather than
//! hard-coding `TcpStream` is what lets `tests/redis_client.rs` script a server —
//! including one that answers half a reply and then dies — with no socket.
//!
//! Timeouts are *not* a method here. They are applied once, when the socket is
//! created (see `client_connect.rs`), so every read and write inherits them and no
//! call site can forget one.

use std::io::{Read, Write};
use std::net::TcpStream;

use super::error::ClientError;

/// A blocking, bidirectional byte stream carrying one Redis session.
///
/// Object-safe: a connection holds a `Box<dyn Transport>` exactly as
/// [`crate::postgres`] holds a boxed socket, so plain TCP and a test double are
/// used identically once the connection exists.
pub trait Transport {
    /// Write every byte of `bytes`, then flush.
    ///
    /// # Arguments
    ///
    /// * `bytes` — A complete encoded command.
    ///
    /// # Errors
    ///
    /// [`ClientError::Transport`] on a short write, a closed peer, or the write
    /// timeout elapsing.
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), ClientError>;

    /// Read up to `into.len()` bytes.
    ///
    /// # Arguments
    ///
    /// * `into` — Destination buffer.
    ///
    /// # Returns
    ///
    /// The number of bytes read. `Ok(0)` means the peer closed the stream, which
    /// the exchange loop treats as a transport failure when a reply is still
    /// outstanding.
    ///
    /// # Errors
    ///
    /// [`ClientError::Transport`] on an I/O failure or the read timeout elapsing.
    fn read(&mut self, into: &mut [u8]) -> Result<usize, ClientError>;
}

/// The production transport. A timeout surfaces here as an `Err`, never a hang.
impl Transport for TcpStream {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), ClientError> {
        Write::write_all(self, bytes)
            .map_err(|error| ClientError::Transport(format!("write command: {error}")))?;
        self.flush()
            .map_err(|error| ClientError::Transport(format!("flush command: {error}")))
    }

    fn read(&mut self, into: &mut [u8]) -> Result<usize, ClientError> {
        Read::read(self, into)
            .map_err(|error| ClientError::Transport(format!("read reply: {error}")))
    }
}
