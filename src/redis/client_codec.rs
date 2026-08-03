//! # The RESP codec boundary
//!
//! This client never calls a concrete codec. It calls [`RespCodec`], whose whole
//! surface is *serialise a command* and *try to frame one reply*. That is the
//! entire contract the real `src/resp/codec.rs` has to satisfy, through a
//! newtype adapter the integrator writes.
//!
//! ## What an adapter must do
//!
//! ```rust,ignore
//! use tetherscript::redis::client::{ClientError, Reply, RespCodec};
//! use tetherscript::resp::codec::{self, DecodeError};
//!
//! pub struct RespAdapter;
//!
//! impl RespCodec for RespAdapter {
//!     fn encode_command(&self, args: &[&[u8]]) -> Result<Vec<u8>, ClientError> {
//!         Ok(codec::encode_command(args))
//!     }
//!
//!     fn decode_reply(&self, buf: &[u8]) -> Result<Option<(Reply, usize)>, ClientError> {
//!         match codec::decode(buf) {
//!             Ok((reply, used)) => Ok(Some((map(reply), used))),
//!             Err(DecodeError::Incomplete) => Ok(None),
//!             Err(other) => Err(ClientError::Protocol(other.to_string())),
//!         }
//!     }
//! }
//! ```
//!
//! Three obligations are load-bearing, because the exchange loop and the pool's
//! discard rule are built on them:
//!
//! 1. **`decode_reply` borrows, never drains.** It returns how many *leading*
//!    bytes the reply used; bytes beyond that belong to the next reply.
//! 2. **Incomplete is `Ok(None)`, not an error, and consumes nothing.** The
//!    caller reads more bytes and retries with the same buffer, grown.
//! 3. **Malformed input is [`ClientError::Protocol`].** It is fatal to the
//!    connection; framing depended on the bytes that were wrong.
//!
//! A `-ERR …` reply is *not* an error here: it decodes to [`Reply::Error`]. Only
//! the exchange layer decides that a top-level one becomes
//! [`ClientError::Server`], and it keeps the connection.

use super::error::ClientError;
use super::reply::Reply;

/// Serialises commands and frames replies for a [`Connection`](super::connection::Connection).
///
/// Implemented by an adapter over the real RESP codec, and by the test double in
/// `tests/redis_client.rs`. Object-safe on purpose: a connection stores a
/// `Box<dyn RespCodec>` so the codec choice is not a generic parameter smeared
/// across every caller.
pub trait RespCodec {
    /// Serialise a command as a RESP array of bulk strings.
    ///
    /// # Arguments
    ///
    /// * `args` — Command name first, then arguments, each as raw bytes.
    ///
    /// # Returns
    ///
    /// The exact bytes to write to the socket.
    ///
    /// # Errors
    ///
    /// [`ClientError::Protocol`] when `args` is empty or an argument exceeds the
    /// codec's bulk-length bound.
    fn encode_command(&self, args: &[&[u8]]) -> Result<Vec<u8>, ClientError>;

    /// Frame the first complete reply at the front of `buf`.
    ///
    /// # Arguments
    ///
    /// * `buf` — Bytes received so far; borrowed, never modified.
    ///
    /// # Returns
    ///
    /// `Some((reply, consumed))` where `consumed` counts the leading bytes the
    /// reply occupied, or `None` when `buf` holds only a valid prefix.
    ///
    /// # Errors
    ///
    /// [`ClientError::Protocol`] when `buf` is not decodable RESP.
    fn decode_reply(&self, buf: &[u8]) -> Result<Option<(Reply, usize)>, ClientError>;
}
