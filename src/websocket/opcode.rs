//! The four-bit WebSocket opcode field (RFC 6455 §5.2).
//!
//! Only the six opcodes the RFC defines are representable. The remaining ten
//! bit patterns are *reserved*, and this type deliberately has no variant for
//! them: an unknown opcode becomes `None` at parse time and the decoder turns
//! that into a protocol violation. Silently ignoring a reserved opcode would let
//! a peer smuggle a frame past the codec that a future extension gives meaning
//! to, so refusing to represent it is the safer shape.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::opcode::Opcode;
//!
//! assert_eq!(Opcode::from_bits(0x1), Some(Opcode::Text));
//! assert_eq!(Opcode::from_bits(0x8), Some(Opcode::Close));
//! // 0x3 is reserved for a future non-control frame.
//! assert_eq!(Opcode::from_bits(0x3), None);
//! assert!(Opcode::Ping.is_control());
//! assert!(!Opcode::Binary.is_control());
//! ```

/// A defined WebSocket frame opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    /// `0x0` — continues the payload of a fragmented message.
    Continuation,
    /// `0x1` — UTF-8 text payload.
    Text,
    /// `0x2` — arbitrary binary payload.
    Binary,
    /// `0x8` — connection close, with an optional code and reason.
    Close,
    /// `0x9` — keepalive probe.
    Ping,
    /// `0xA` — reply to a [`Opcode::Ping`].
    Pong,
}

impl Opcode {
    /// Decode the low four bits of a frame's first byte.
    ///
    /// # Arguments
    ///
    /// * `bits` — Candidate opcode. Only the low four bits are inspected.
    ///
    /// # Returns
    ///
    /// `Some(opcode)` for a defined opcode, `None` for a reserved one.
    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits & 0x0f {
            0x0 => Some(Self::Continuation),
            0x1 => Some(Self::Text),
            0x2 => Some(Self::Binary),
            0x8 => Some(Self::Close),
            0x9 => Some(Self::Ping),
            0xa => Some(Self::Pong),
            _ => None,
        }
    }

    /// The wire encoding of this opcode.
    ///
    /// # Returns
    ///
    /// A value in `0x0..=0xA` suitable for the low nibble of byte 0.
    pub fn to_bits(self) -> u8 {
        match self {
            Self::Continuation => 0x0,
            Self::Text => 0x1,
            Self::Binary => 0x2,
            Self::Close => 0x8,
            Self::Ping => 0x9,
            Self::Pong => 0xa,
        }
    }

    /// Whether this is a control opcode, which may not be fragmented and whose
    /// payload may not exceed 125 bytes.
    ///
    /// # Returns
    ///
    /// `true` for close, ping, and pong.
    pub fn is_control(self) -> bool {
        matches!(self, Self::Close | Self::Ping | Self::Pong)
    }
}
