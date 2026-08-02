//! Transport abstraction for the PostgreSQL socket.
//!
//! A connection is either a plain TCP stream or a TLS stream negotiated over one.
//! Both are used identically after the handshake, so the protocol code works
//! through this trait object rather than being generic over the stream type.

use std::io::{Read, Write};

/// A readable, writable PostgreSQL socket.
///
/// Blanket-implemented, so any `Read + Write` type qualifies.
pub(super) trait Transport: Read + Write {}

impl<T: Read + Write> Transport for T {}

/// The socket a [`Connection`](super::connection::Connection) owns.
pub(super) type Socket = Box<dyn Transport>;
