//! Which endpoint sent the frame being decoded.
//!
//! RFC 6455 §5.1 makes masking *directional*, not optional: "the client MUST
//! mask all frames" and "a server MUST NOT mask any frames". A decoder that does
//! not know which side it is reading cannot enforce either half, so the role is a
//! required argument rather than a default. Passing the wrong role would turn a
//! security check into a no-op, which is why there is no `Default` impl.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::role::Role;
//!
//! // A server reads frames sent by a client, so it requires masking.
//! assert!(Role::Client.requires_mask());
//! assert!(!Role::Server.requires_mask());
//! ```

/// The originator of a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The frame was sent by a client; MASK must be set.
    Client,
    /// The frame was sent by a server; MASK must be clear.
    Server,
}

impl Role {
    /// Whether frames from this role must carry a masking key.
    ///
    /// # Returns
    ///
    /// `true` for [`Role::Client`], `false` for [`Role::Server`].
    pub fn requires_mask(self) -> bool {
        matches!(self, Self::Client)
    }
}
