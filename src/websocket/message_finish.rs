//! Converting a fully reassembled fragment buffer into a [`Message`].
//!
//! UTF-8 is validated here rather than per-fragment: a multi-byte character may
//! straddle a fragment boundary, so a per-fragment check would reject valid
//! traffic. Waiting until the join is the only correct place, and it is bounded
//! because the buffer size was already capped by
//! [`crate::websocket::limits::check_message`].

use crate::websocket::error::ProtocolError;
use crate::websocket::message::Message;
use crate::websocket::opcode::Opcode;
use crate::websocket::validate;

/// Build the finished message for the opcode that opened it.
///
/// # Arguments
///
/// * `kind` — The opcode of the first fragment: `Text` or `Binary`.
/// * `bytes` — The joined payload.
///
/// # Returns
///
/// [`Message::Text`] for a text message, [`Message::Binary`] otherwise.
///
/// # Errors
///
/// [`ProtocolError::InvalidUtf8`] when a text message is not valid UTF-8.
///
/// # Panics
///
/// Never. UTF-8 validity is established before `from_utf8`, and the error branch
/// returns rather than unwrapping, so no decode failure can reach a panic.
pub(super) fn finish(kind: Opcode, bytes: Vec<u8>) -> Result<Message, ProtocolError> {
    if kind != Opcode::Text {
        return Ok(Message::Binary(bytes));
    }
    validate::utf8(&bytes, "text message")?;
    match String::from_utf8(bytes) {
        Ok(text) => Ok(Message::Text(text)),
        Err(_) => Err(ProtocolError::InvalidUtf8 {
            context: "text message",
        }),
    }
}
