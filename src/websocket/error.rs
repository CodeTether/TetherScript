//! Protocol violations the codec reports instead of panicking.
//!
//! Two things are deliberately distinct here. [`ProtocolError`] means *these
//! bytes can never become a valid frame* — the connection must fail. It is a
//! separate type from the "not yet" case, which [`crate::websocket::frame`]
//! models as [`DecodeOutcome::Incomplete`](crate::websocket::frame::DecodeOutcome).
//! Conflating the two is the classic streaming-parser bug: a peer that writes a
//! frame across two TCP segments would be treated as hostile, or worse, a
//! genuinely malformed frame would be treated as "wait for more" and stall the
//! connection forever while the read buffer grows.
//!
//! Every variant names the offending value, per the repository's error-message
//! rule, so an operator reading a log can tell *which* bound a peer tripped.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::error::ProtocolError;
//!
//! let err = ProtocolError::ReservedOpcode { bits: 0x3 };
//! assert!(err.to_string().contains("0x3"));
//! ```

use std::fmt;

/// A fatal WebSocket protocol violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    /// An opcode outside the six defined by RFC 6455 §5.2.
    ReservedOpcode {
        /// The reserved four-bit value that was received.
        bits: u8,
    },
    /// One or more of RSV1/RSV2/RSV3 was set with no extension negotiated.
    ReservedBitSet {
        /// The three reserved bits, right-aligned in `0..=7`.
        rsv: u8,
    },
    /// A client-to-server frame arrived with MASK clear.
    UnmaskedClientFrame,
    /// A server-to-client frame arrived with MASK set.
    MaskedServerFrame,
    /// A control frame carried FIN clear.
    FragmentedControlFrame,
    /// A control frame payload exceeded 125 bytes.
    ControlPayloadTooLarge {
        /// The declared control payload length.
        len: u64,
    },
    /// The 64-bit length form had its most significant bit set.
    LengthMsbSet {
        /// The raw 64-bit value as it appeared on the wire.
        raw: u64,
    },
    /// A shorter length form would have encoded this value (§5.2 minimality).
    NonMinimalLength {
        /// The value that was encoded too widely.
        len: u64,
    },
    /// The declared frame payload exceeded the configured bound.
    PayloadTooLarge {
        /// Length the peer declared.
        declared: u64,
        /// Bound that was exceeded.
        max: u64,
    },
    /// A reassembled message exceeded the configured bound.
    MessageTooLarge {
        /// Bytes buffered so far.
        total: usize,
        /// Bound that was exceeded.
        max: usize,
    },
    /// A text payload or close reason was not valid UTF-8.
    InvalidUtf8 {
        /// Where the invalid sequence was found, for the log line.
        context: &'static str,
    },
    /// A close frame carried a body of exactly one byte.
    TruncatedCloseCode,
    /// A close code that must never appear on the wire, or is unassigned.
    ForbiddenCloseCode {
        /// The code that was received.
        code: u16,
    },
    /// A continuation frame arrived with no message in progress.
    UnexpectedContinuation,
    /// A new data frame arrived while a fragmented message was in progress.
    InterleavedDataFrame,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::websocket::error_text::write(self, f)
    }
}

impl std::error::Error for ProtocolError {}
