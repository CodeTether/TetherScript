//! Inspection and formatting for [`RedisError`].
//!
//! Split from the enum definition so the type's documentation stays readable and
//! each file keeps one responsibility. Every message is prefixed `redis:` to
//! match how [`crate::postgres`] names the subsystem that failed.

use std::fmt;

use super::error::RedisError;

impl RedisError {
    /// Whether this is an error *reply* rather than a transport or framing fault.
    ///
    /// # Returns
    ///
    /// `true` only for [`RedisError::Server`], where the round trip succeeded and
    /// the connection may keep being used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::RedisError;
    ///
    /// let reply = RedisError::Server { kind: "ERR".into(), message: "bad".into() };
    /// assert!(reply.is_server_error());
    /// assert!(!RedisError::Protocol("short".into()).is_server_error());
    /// ```
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Server { .. })
    }

    /// The server's error kind, such as `ERR`, `WRONGTYPE`, or `NOAUTH`.
    ///
    /// # Returns
    ///
    /// `Some(kind)` for [`RedisError::Server`], `None` for every other variant,
    /// because no other variant came from the server.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::RedisError;
    ///
    /// let reply = RedisError::Server { kind: "NOAUTH".into(), message: "x".into() };
    /// assert_eq!(reply.kind(), Some("NOAUTH"));
    /// ```
    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Server { kind, .. } => Some(kind.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(detail) => write!(f, "redis: transport: {detail}"),
            Self::Protocol(detail) => write!(f, "redis: protocol: {detail}"),
            Self::Server { kind, message } => write!(f, "redis: server: {kind} {message}"),
            Self::UnexpectedType(detail) => write!(f, "redis: unexpected reply: {detail}"),
        }
    }
}

impl std::error::Error for RedisError {}
