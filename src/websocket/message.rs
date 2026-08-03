//! Fragment reassembly (RFC 6455 §5.4).
//!
//! A message is a data frame with FIN clear, zero or more `Continuation` frames
//! with FIN clear, then a `Continuation` with FIN set. Control frames may be
//! *interleaved* into that sequence and are passed straight through, because a
//! ping must be answerable while a large message is still arriving.
//!
//! Three sequencing rules are enforced:
//!
//! * A `Continuation` with no message in progress is rejected — there is nothing
//!   to continue, and silently starting a message would let a peer inject bytes
//!   with no declared type.
//! * A new `Text`/`Binary` frame while a message is in progress is rejected;
//!   interleaving data messages is not permitted.
//! * The running total is checked against
//!   [`crate::websocket::limits::MAX_MESSAGE_LEN`] after every fragment, so a
//!   peer cannot exhaust memory with an unbounded number of small in-bound
//!   fragments even though each individual frame is within the frame bound.
//!
//! UTF-8 validation for text is applied to the *joined* payload, since a
//! multi-byte character may legally straddle a fragment boundary.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::frame::Frame;
//! use tetherscript::websocket::message::{Assembler, Message};
//! use tetherscript::websocket::opcode::Opcode;
//!
//! let mut assembler = Assembler::new();
//! let head = Frame { fin: false, opcode: Opcode::Text, payload: b"he".to_vec() };
//! let tail = Frame { fin: true, opcode: Opcode::Continuation, payload: b"llo".to_vec() };
//! assert_eq!(assembler.accept(head).unwrap(), None);
//! assert_eq!(
//!     assembler.accept(tail).unwrap(),
//!     Some(Message::Text("hello".into())),
//! );
//! ```

use crate::websocket::opcode::Opcode;

/// A completed application message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// A complete, UTF-8 validated text message.
    Text(String),
    /// A complete binary message.
    Binary(Vec<u8>),
    /// A close frame, with its parsed body when one was present.
    Close(Option<crate::websocket::close::CloseFrame>),
    /// A ping, with its payload (at most 125 bytes).
    Ping(Vec<u8>),
    /// A pong, with its payload (at most 125 bytes).
    Pong(Vec<u8>),
}

/// Accumulates fragments until a message is complete.
#[derive(Debug, Default)]
pub struct Assembler {
    /// The opcode that opened the message in progress, if any.
    pub(super) started: Option<Opcode>,
    /// Buffered fragment bytes for the message in progress.
    pub(super) buffer: Vec<u8>,
}

impl Assembler {
    /// Create an assembler with no message in progress.
    ///
    /// # Returns
    ///
    /// A fresh [`Assembler`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one decoded frame into the assembler.
    ///
    /// # Arguments
    ///
    /// * `frame` — A frame already validated by [`crate::websocket::frame`].
    ///
    /// # Returns
    ///
    /// `Ok(Some(message))` when this frame completed a message (control frames
    /// always complete immediately), or `Ok(None)` when more fragments are
    /// expected.
    ///
    /// # Errors
    ///
    /// [`ProtocolError`](crate::websocket::error::ProtocolError) for a
    /// sequencing violation, an oversized message, or invalid joined UTF-8.
    pub fn accept(
        &mut self,
        frame: crate::websocket::frame::Frame,
    ) -> Result<Option<Message>, crate::websocket::error::ProtocolError> {
        crate::websocket::message_accept::accept(self, frame)
    }
}
