//! Failure kinds for the native Redis client.
//!
//! The distinction this module exists to preserve: **an error reply is a
//! protocol-level success**. When Redis answers `-WRONGTYPE Operation against a
//! key holding the wrong kind of value`, the transport worked, the framing was
//! valid, and the server understood the request. Conflating that with a broken
//! socket would make a retry loop hammer a connection that is perfectly healthy,
//! and would hide the fact that the *command* was wrong. So
//! [`RedisError::Server`] is its own variant, carries the server's error kind
//! separately from its message, and is never produced by the socket layer.
//!
//! Inspection and `Display` live in the `error_impl` module, keeping this file to
//! the single job of naming the failure kinds.

/// Everything that can go wrong talking to Redis.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis::RedisError;
///
/// let server = RedisError::Server {
///     kind: "WRONGTYPE".into(),
///     message: "Operation against a key holding the wrong kind of value".into(),
/// };
/// assert_eq!(server.kind(), Some("WRONGTYPE"));
/// assert!(server.is_server_error());
///
/// let broken = RedisError::Transport("connection reset by peer".into());
/// assert!(!broken.is_server_error());
/// assert_eq!(broken.kind(), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedisError {
    /// The socket failed: connect, read, write, timeout, or a clean EOF where a
    /// reply was still owed. Nothing is known about whether the command ran.
    Transport(String),
    /// The bytes on the wire were not valid RESP, or declared a length past the
    /// documented maximum. The connection's framing is now unknown, so a caller
    /// must discard it rather than send another command.
    Protocol(String),
    /// Redis replied `-<kind> <message>`. The exchange succeeded; the command
    /// did not. The connection remains usable.
    Server {
        /// Leading token of the error reply, such as `ERR` or `WRONGTYPE`.
        kind: String,
        /// Human-readable remainder of the error reply.
        message: String,
    },
    /// A reply arrived intact but was not the RESP type the command expects,
    /// naming both the command and what came back.
    UnexpectedType(String),
}
