//! Human-readable rendering of [`ProtocolError`].
//!
//! Split out from `error.rs` so the variant list and its prose live in separate
//! files: the enum is the protocol's vocabulary, this is its diagnostics. Each
//! message names the concrete offending value, because "protocol error" tells an
//! operator nothing about which peer did what.

use crate::websocket::error::ProtocolError;
use std::fmt;

/// Write the operator-facing description of `error`.
///
/// # Arguments
///
/// * `error` — The violation to describe.
/// * `f` — Formatter supplied by the [`fmt::Display`] impl.
///
/// # Returns
///
/// Whatever the underlying formatter returns.
pub(super) fn write(error: &ProtocolError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        ProtocolError::ReservedOpcode { bits } => {
            write!(f, "websocket: reserved opcode 0x{bits:x}")
        }
        ProtocolError::ReservedBitSet { rsv } => {
            write!(f, "websocket: reserved bits set (RSV=0b{rsv:03b})")
        }
        ProtocolError::UnmaskedClientFrame => {
            write!(f, "websocket: client frame is not masked")
        }
        ProtocolError::MaskedServerFrame => {
            write!(f, "websocket: server frame must not be masked")
        }
        ProtocolError::FragmentedControlFrame => {
            write!(f, "websocket: control frame must not be fragmented")
        }
        ProtocolError::ControlPayloadTooLarge { len } => {
            write!(f, "websocket: control payload {len} bytes exceeds 125")
        }
        ProtocolError::LengthMsbSet { raw } => {
            write!(f, "websocket: 64-bit length 0x{raw:016x} has MSB set")
        }
        ProtocolError::NonMinimalLength { len } => {
            write!(f, "websocket: length {len} used a non-minimal encoding")
        }
        _ => crate::websocket::error_text_more::write(error, f),
    }
}
