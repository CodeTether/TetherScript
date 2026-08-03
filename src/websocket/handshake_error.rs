//! Handshake failures, kept separate from frame-level [`ProtocolError`].
//!
//! A handshake failure is answered with an HTTP error response, not a close
//! frame, because the connection is still HTTP at that point — the two failure
//! modes have different remediations and so they get different types rather than
//! one stringly-typed error.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::handshake_error::HandshakeError;
//!
//! let err = HandshakeError::UnsupportedVersion { version: "8".into() };
//! assert!(err.to_string().contains("8"));
//! ```

use std::fmt;

/// A rejected WebSocket opening handshake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeError {
    /// A required header was absent.
    MissingHeader {
        /// The header name, lowercased.
        name: &'static str,
    },
    /// `Upgrade` was present but did not name `websocket`.
    BadUpgrade {
        /// The value that was received.
        value: String,
    },
    /// `Connection` did not include the `Upgrade` token.
    BadConnection {
        /// The value that was received.
        value: String,
    },
    /// `Sec-WebSocket-Version` was not `13`.
    UnsupportedVersion {
        /// The version that was requested.
        version: String,
    },
    /// `Sec-WebSocket-Key` was not 16 bytes of base64.
    BadKey {
        /// Why the key was rejected.
        reason: String,
    },
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHeader { name } => {
                write!(f, "websocket handshake: missing `{name}` header")
            }
            Self::BadUpgrade { value } => {
                write!(
                    f,
                    "websocket handshake: Upgrade is `{value}`, want `websocket`"
                )
            }
            Self::BadConnection { value } => {
                write!(f, "websocket handshake: Connection `{value}` lacks Upgrade")
            }
            Self::UnsupportedVersion { version } => {
                write!(
                    f,
                    "websocket handshake: version `{version}` unsupported, want 13"
                )
            }
            Self::BadKey { reason } => {
                write!(f, "websocket handshake: Sec-WebSocket-Key {reason}")
            }
        }
    }
}

impl std::error::Error for HandshakeError {}
