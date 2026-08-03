//! The error taxonomy for the Redis client, and the discard verdict it carries.
//!
//! The split between variants is not cosmetic: it is what the pool consults to
//! decide whether a connection may be handed out again.
//!
//! - [`ClientError::Transport`] and [`ClientError::Protocol`] abandon the exchange
//!   mid-stream. Unread or unparsable bytes may still be queued, so the next
//!   command on that socket would read this command's leftovers and every later
//!   reply would be off by one. The connection must be dropped.
//! - [`ClientError::Server`] and [`ClientError::UnexpectedType`] arrive as a fully
//!   framed reply. One reply was consumed for one request, so the stream is still
//!   aligned and the connection is reusable.
//!
//! `src/postgres/pool.rs` draws the same line: a query answered with an
//! `ErrorResponse` is released, a query that dies in the transport is discarded.

use std::fmt;

/// Everything that can go wrong between a caller and a Redis server.
///
/// # Examples
///
/// ```rust,ignore
/// use tetherscript::redis::client::ClientError;
///
/// let server = ClientError::Server { kind: "WRONGTYPE".into(), message: "…".into() };
/// assert!(!server.discards_connection());
/// assert!(ClientError::Transport("timed out".into()).discards_connection());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    /// The socket failed: connect, write, read, timeout, or an unexpected EOF.
    Transport(String),
    /// The bytes on the wire were not decodable RESP.
    Protocol(String),
    /// The server answered with an error reply, such as `-WRONGTYPE …`. The
    /// exchange *completed*: this is data, not a fault.
    Server {
        /// Leading token of the error line, e.g. `ERR` or `WRONGPASS`.
        kind: String,
        /// Remainder of the error line.
        message: String,
    },
    /// A well-formed reply of the wrong RESP type for the command that asked.
    UnexpectedType(String),
    /// Every connection the pool may own is leased out.
    PoolExhausted {
        /// Connections currently owned by the pool.
        in_use: usize,
        /// The configured ceiling, named so the fix is visibly "raise the limit".
        max: usize,
    },
}

impl ClientError {
    /// Whether the connection that produced this error must be thrown away.
    ///
    /// # Returns
    ///
    /// `true` for `Transport` and `Protocol`; `false` for `Server`,
    /// `UnexpectedType`, and `PoolExhausted` (which never held a connection).
    pub fn discards_connection(&self) -> bool {
        matches!(self, Self::Transport(_) | Self::Protocol(_))
    }

    /// Whether this is a server-side error reply rather than a client fault.
    ///
    /// # Returns
    ///
    /// `true` only for [`ClientError::Server`].
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Server { .. })
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(detail) => write!(f, "redis: transport: {detail}"),
            Self::Protocol(detail) => write!(f, "redis: protocol: {detail}"),
            Self::Server { kind, message } => write!(f, "redis: server: {kind} {message}"),
            Self::UnexpectedType(detail) => write!(f, "redis: unexpected reply: {detail}"),
            Self::PoolExhausted { in_use, max } => write!(
                f,
                "redis: connection pool exhausted ({in_use} in use, max {max})"
            ),
        }
    }
}

impl std::error::Error for ClientError {}
