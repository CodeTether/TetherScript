//! The request/reply exchange: write a command, frame exactly one reply.
//!
//! This is the only place bytes move, so it is also the only place that can leave
//! a connection mid-stream. Every error it returns is classified so the pool can
//! act on it; see `client_error.rs` for the discard rule.

use super::connection::Connection;
use super::error::ClientError;
use super::reply::Reply;

/// Read granularity. Large enough that a typical reply arrives in one read.
const CHUNK: usize = 8 * 1024;

impl Connection {
    /// Encode `args`, send them, and return the server's reply.
    ///
    /// This is the generic escape hatch: any command Redis understands can be
    /// issued through it, including ones with no typed method here.
    ///
    /// # Arguments
    ///
    /// * `args` — Command name first, then arguments, each as raw bytes.
    ///
    /// # Returns
    ///
    /// The decoded reply. A top-level [`Reply::Error`] is *not* returned; it is
    /// promoted to [`ClientError::Server`], and the connection stays usable
    /// because one whole reply was consumed for one whole request.
    ///
    /// # Errors
    ///
    /// [`ClientError::Protocol`] when `args` is unencodable or the reply is not
    /// RESP; [`ClientError::Transport`] on any socket failure, including the peer
    /// closing with a reply outstanding; [`ClientError::Server`] for an error
    /// reply.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pong = connection.command(&[b"PING"])?;
    /// ```
    pub fn command(&mut self, args: &[&[u8]]) -> Result<Reply, ClientError> {
        let request = self.codec.encode_command(args)?;
        self.transport.write_all(&request)?;
        match self.read_reply()? {
            Reply::Error { kind, message } => Err(ClientError::Server { kind, message }),
            other => Ok(other),
        }
    }

    /// Frame one reply, reading more bytes until the codec has enough.
    fn read_reply(&mut self) -> Result<Reply, ClientError> {
        loop {
            let framed = self.codec.decode_reply(&self.buffer)?;
            if let Some((reply, consumed)) = framed {
                // Drain exactly what the reply used: the rest is the next reply.
                self.buffer.drain(..consumed);
                return Ok(reply);
            }
            let mut chunk = [0u8; CHUNK];
            match self.transport.read(&mut chunk)? {
                0 => {
                    return Err(ClientError::Transport(
                        "server closed the connection with a reply outstanding".into(),
                    ));
                }
                read => self.buffer.extend_from_slice(&chunk[..read]),
            }
        }
    }
}
