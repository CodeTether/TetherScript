//! The fragment state machine driving [`Assembler::accept`].
//!
//! Split from `message.rs` so the public message vocabulary and the transition
//! rules are separately readable. On any error the buffer is *not* cleared,
//! because a sequencing violation is fatal to the connection: the caller must
//! close, not resume.

use crate::websocket::error::ProtocolError;
use crate::websocket::frame::Frame;
use crate::websocket::limits;
use crate::websocket::message::{Assembler, Message};
use crate::websocket::message_finish::finish;
use crate::websocket::opcode::Opcode;

/// Apply one frame to the assembler state.
///
/// # Arguments
///
/// * `state` — The assembler being driven.
/// * `frame` — A frame already validated at the framing layer.
///
/// # Returns
///
/// `Ok(Some(message))` on completion, `Ok(None)` while fragments remain.
///
/// # Errors
///
/// [`ProtocolError::UnexpectedContinuation`],
/// [`ProtocolError::InterleavedDataFrame`], [`ProtocolError::MessageTooLarge`],
/// or [`ProtocolError::InvalidUtf8`].
pub(super) fn accept(
    state: &mut Assembler,
    frame: Frame,
) -> Result<Option<Message>, ProtocolError> {
    if frame.opcode.is_control() {
        return control(frame).map(Some);
    }
    if frame.opcode == Opcode::Continuation && state.started.is_none() {
        return Err(ProtocolError::UnexpectedContinuation);
    }
    if frame.opcode != Opcode::Continuation && state.started.is_some() {
        return Err(ProtocolError::InterleavedDataFrame);
    }
    let kind = *state.started.get_or_insert(frame.opcode);
    state.buffer.extend_from_slice(&frame.payload);
    limits::check_message(state.buffer.len())?;
    if !frame.fin {
        return Ok(None);
    }
    let bytes = std::mem::take(&mut state.buffer);
    state.started = None;
    finish(kind, bytes).map(Some)
}

/// Turn a control frame into its message form.
///
/// # Arguments
///
/// * `frame` — A control frame; its payload was already bounded to 125 bytes and,
///   for close, validated by [`crate::websocket::close`].
///
/// # Returns
///
/// The corresponding [`Message`].
///
/// # Errors
///
/// Propagates a close-body error, though the decoder has already rejected those.
fn control(frame: Frame) -> Result<Message, ProtocolError> {
    match frame.opcode {
        Opcode::Ping => Ok(Message::Ping(frame.payload)),
        Opcode::Pong => Ok(Message::Pong(frame.payload)),
        // Only Close remains: `is_control` admits exactly Ping, Pong, and Close,
        // so this arm is total without a panicking catch-all.
        _ => crate::websocket::close::validate(&frame.payload).map(Message::Close),
    }
}
