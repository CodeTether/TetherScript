//! Resource bounds for the WebSocket codec.
//!
//! Every byte this codec parses arrives from a remote peer, and RFC 6455 lets a
//! peer *declare* a payload length of up to 2^63 - 1 before sending a single
//! byte of it. Trusting that number would let one 10-byte frame header ask for a
//! multi-exabyte allocation. So the declared length is checked against a bound
//! **before** any buffer is sized from it.
//!
//! ## The bounds
//!
//! | Bound | Value | Why |
//! |---|---|---|
//! | [`MAX_PAYLOAD_LEN`] | 16 MiB | Largest single frame payload accepted. |
//! | [`MAX_MESSAGE_LEN`] | 64 MiB | Largest reassembled message across fragments. |
//! | [`MAX_CONTROL_PAYLOAD_LEN`] | 125 | Fixed by RFC 6455 §5.5, not a policy choice. |
//!
//! `MAX_PAYLOAD_LEN` also makes the `u64 as usize` narrowing in the decoder
//! lossless on 32-bit targets: a value that survives the check is far below
//! `u32::MAX`, so the cast cannot silently wrap into a small length.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::limits;
//!
//! assert!(limits::check_payload(1024).is_ok());
//! assert!(limits::check_payload(u64::MAX / 2).is_err());
//! ```

use crate::websocket::error::ProtocolError;

/// Largest accepted payload for one frame, in bytes (16 MiB).
pub const MAX_PAYLOAD_LEN: u64 = 16 * 1024 * 1024;

/// Largest accepted reassembled message, in bytes (64 MiB).
pub const MAX_MESSAGE_LEN: usize = 64 * 1024 * 1024;

/// Largest control-frame payload, in bytes. Fixed by RFC 6455 §5.5.
pub const MAX_CONTROL_PAYLOAD_LEN: usize = 125;

/// Reject a declared frame payload length that exceeds [`MAX_PAYLOAD_LEN`].
///
/// # Arguments
///
/// * `declared` — The length decoded from the frame's length field.
///
/// # Returns
///
/// `Ok(())` when `declared` is within bounds.
///
/// # Errors
///
/// [`ProtocolError::PayloadTooLarge`] when the peer declared more than the bound.
pub fn check_payload(declared: u64) -> Result<(), ProtocolError> {
    if declared > MAX_PAYLOAD_LEN {
        return Err(ProtocolError::PayloadTooLarge {
            declared,
            max: MAX_PAYLOAD_LEN,
        });
    }
    Ok(())
}

/// Reject a reassembled message that exceeds [`MAX_MESSAGE_LEN`].
///
/// # Arguments
///
/// * `total` — Running total of buffered fragment bytes.
///
/// # Returns
///
/// `Ok(())` when `total` is within bounds.
///
/// # Errors
///
/// [`ProtocolError::MessageTooLarge`] when the fragment sequence grew too big.
pub fn check_message(total: usize) -> Result<(), ProtocolError> {
    if total > MAX_MESSAGE_LEN {
        return Err(ProtocolError::MessageTooLarge {
            total,
            max: MAX_MESSAGE_LEN,
        });
    }
    Ok(())
}
