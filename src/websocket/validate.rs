//! Payload-level validation applied after unmasking.
//!
//! Two payloads carry structure the codec must check before the application sees
//! them:
//!
//! * **Text.** RFC 6455 §8.1 requires text payloads to be valid UTF-8. This is
//!   checked here only for an unfragmented text frame (`fin` set); for a
//!   fragmented message, a multi-byte character may legally straddle the
//!   fragment boundary, so validation is deferred to
//!   [`crate::websocket::message`] once the fragments are joined.
//! * **Close.** The status code and reason are validated in
//!   [`crate::websocket::close`], because a forbidden code such as 1006 must be
//!   refused at the wire rather than surfaced to the application as if a peer had
//!   really sent it.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::opcode::Opcode;
//! use tetherscript::websocket::validate;
//!
//! assert!(validate::payload(Opcode::Text, true, b"ok").is_ok());
//! // A lone 0x80 continuation byte is not valid UTF-8.
//! assert!(validate::payload(Opcode::Text, true, &[0x80]).is_err());
//! // The same bytes are tolerated mid-message and checked at reassembly.
//! assert!(validate::payload(Opcode::Text, false, &[0x80]).is_ok());
//! ```

use crate::websocket::close;
use crate::websocket::error::ProtocolError;
use crate::websocket::opcode::Opcode;

/// Validate a decoded, unmasked payload for its opcode.
///
/// # Arguments
///
/// * `opcode` — The frame's opcode.
/// * `fin` — Whether this frame completes its message.
/// * `payload` — The unmasked payload bytes.
///
/// # Returns
///
/// `Ok(())` when the payload is acceptable for this opcode.
///
/// # Errors
///
/// [`ProtocolError::InvalidUtf8`] for a bad complete text payload, or any close
/// frame error from [`close::validate`].
pub fn payload(opcode: Opcode, fin: bool, payload: &[u8]) -> Result<(), ProtocolError> {
    match opcode {
        Opcode::Text if fin => utf8(payload, "text payload"),
        Opcode::Close => close::validate(payload).map(|_| ()),
        _ => Ok(()),
    }
}

/// Reject `bytes` unless they are valid UTF-8.
///
/// # Arguments
///
/// * `bytes` — Candidate UTF-8.
/// * `context` — Names the field for the error message.
///
/// # Returns
///
/// `Ok(())` when `bytes` decodes cleanly.
///
/// # Errors
///
/// [`ProtocolError::InvalidUtf8`] carrying `context`.
pub fn utf8(bytes: &[u8], context: &'static str) -> Result<(), ProtocolError> {
    std::str::from_utf8(bytes)
        .map(|_| ())
        .map_err(|_| ProtocolError::InvalidUtf8 { context })
}
