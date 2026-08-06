//! Whole frames: the [`Frame`] value and the incremental [`DecodeOutcome`].
//!
//! ## Incomplete is not Malformed
//!
//! A stream decoder must answer three questions, not two: is this frame *done*,
//! *broken*, or *not here yet*? [`DecodeOutcome`] keeps the third case separate
//! from `Err`, and on [`DecodeOutcome::Incomplete`] the decoder reports zero
//! bytes consumed, so the caller keeps its buffer intact and retries after the
//! next read. On [`DecodeOutcome::Frame`] it reports exactly how many bytes the
//! frame occupied, which is how a caller drains a buffer that may hold several
//! frames back to back.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::frame::{DecodeOutcome, Frame};
//! use tetherscript::websocket::opcode::Opcode;
//! use tetherscript::websocket::role::Role;
//!
//! // Masked client "Hello" from RFC 6455 §5.7.
//! let wire = [
//!     0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58,
//! ];
//! match Frame::decode(&wire, Role::Client).unwrap() {
//!     DecodeOutcome::Frame { frame, consumed } => {
//!         assert_eq!(consumed, 11);
//!         assert_eq!(frame.opcode, Opcode::Text);
//!         assert_eq!(frame.payload, b"Hello".to_vec());
//!     }
//!     DecodeOutcome::Incomplete => panic!("frame is complete"),
//! }
//!
//! // Every strict prefix is Incomplete, never an error.
//! for cut in 0..wire.len() {
//!     let outcome = Frame::decode(&wire[..cut], Role::Client).unwrap();
//!     assert_eq!(outcome, DecodeOutcome::Incomplete);
//! }
//! ```

use crate::websocket::error::ProtocolError;
use crate::websocket::opcode::Opcode;
use crate::websocket::role::Role;

/// One decoded frame, with its payload already unmasked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// FIN: the final fragment of its message.
    pub fin: bool,
    /// The frame's opcode.
    pub opcode: Opcode,
    /// Payload bytes, unmasked. Never longer than the payload bound.
    pub payload: Vec<u8>,
}

/// The result of one non-consuming decode attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeOutcome {
    /// A complete frame, plus how many input bytes it occupied.
    Frame {
        /// The decoded frame.
        frame: Frame,
        /// Bytes the caller should drain from the front of its buffer.
        consumed: usize,
    },
    /// More bytes are needed. Consume nothing and retry after the next read.
    Incomplete,
}

impl Frame {
    /// Decode one frame from the front of `bytes`.
    ///
    /// # Arguments
    ///
    /// * `bytes` — Read buffer, starting at a frame boundary.
    /// * `role` — Sender of the frame; enforces the masking direction.
    ///
    /// # Returns
    ///
    /// [`DecodeOutcome::Frame`] with the byte count, or
    /// [`DecodeOutcome::Incomplete`].
    ///
    /// # Errors
    ///
    /// Any [`ProtocolError`]; the input is a protocol violation and the
    /// connection must be failed rather than resynchronized.
    ///
    /// # Panics
    ///
    /// Never; see `crate::websocket::decode`.
    pub fn decode(bytes: &[u8], role: Role) -> Result<DecodeOutcome, ProtocolError> {
        crate::websocket::decode::decode(bytes, role)
    }

    /// Encode this frame as a server, i.e. without a masking key.
    ///
    /// # Returns
    ///
    /// The complete wire bytes, header included.
    pub fn encode_server(&self) -> Vec<u8> {
        crate::websocket::encode::encode_server(self.fin, self.opcode, &self.payload)
    }
}
