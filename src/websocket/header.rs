//! The parsed WebSocket frame header (RFC 6455 §5.2).
//!
//! Parsing is *incremental and non-consuming*: a truncated header returns
//! `Ok(None)`, never an error, so a caller may re-parse the same buffer after
//! more bytes arrive. Validation runs as early as each field becomes available,
//! so a hostile header is rejected before its declared payload is ever used to
//! size a buffer.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::header::FrameHeader;
//! use tetherscript::websocket::opcode::Opcode;
//! use tetherscript::websocket::role::Role;
//!
//! // A masked client text frame carrying 5 bytes.
//! let bytes = [0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d];
//! let header = FrameHeader::parse(&bytes, Role::Client).unwrap().unwrap();
//! assert!(header.fin);
//! assert_eq!(header.opcode, Opcode::Text);
//! assert_eq!(header.payload_len, 5);
//! assert_eq!(header.header_len, 6);
//! assert_eq!(header.mask, Some([0x37, 0xfa, 0x21, 0x3d]));
//!
//! // One byte short of the masking key: incomplete, not malformed.
//! assert!(FrameHeader::parse(&bytes[..5], Role::Client).unwrap().is_none());
//! ```

use crate::websocket::error::ProtocolError;
use crate::websocket::header_parse::parse_validated;
use crate::websocket::opcode::Opcode;
use crate::websocket::role::Role;

/// A fully validated frame header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    /// FIN: this is the final fragment of its message.
    pub fin: bool,
    /// The frame's opcode.
    pub opcode: Opcode,
    /// Masking key, present exactly when MASK was set.
    pub mask: Option<[u8; 4]>,
    /// Declared payload length, already checked against the payload bound.
    pub payload_len: usize,
    /// Total header size in bytes, i.e. the offset of the payload.
    pub header_len: usize,
}

impl FrameHeader {
    /// Parse a frame header from the front of `bytes`.
    ///
    /// # Arguments
    ///
    /// * `bytes` — Buffer whose first byte is the frame's first byte.
    /// * `role` — Which endpoint sent the frame, which decides whether MASK is
    ///   required or forbidden.
    ///
    /// # Returns
    ///
    /// `Ok(Some(header))` when the whole header is present, or `Ok(None)` when
    /// more bytes are needed. Nothing is consumed in either case.
    ///
    /// # Errors
    ///
    /// Any [`ProtocolError`] framing variant: reserved opcode or RSV bit, a
    /// masking-direction violation, a fragmented or oversized control frame, a
    /// 64-bit length with its MSB set, a non-minimal length, or a payload beyond
    /// [`crate::websocket::limits::MAX_PAYLOAD_LEN`].
    ///
    /// # Panics
    ///
    /// Never. Every byte is read through `get`, and the mask offsets are derived
    /// from the length-field width (at most 10), not from attacker-chosen values.
    pub fn parse(bytes: &[u8], role: Role) -> Result<Option<Self>, ProtocolError> {
        parse_validated(bytes, role)
    }
}
