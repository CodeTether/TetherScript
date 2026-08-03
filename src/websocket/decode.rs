//! Frame body decoding: slicing the payload and unmasking it.
//!
//! # How out-of-range indexing is ruled out
//!
//! There is exactly one slice of attacker-influenced length in this codec, and it
//! is taken with `get(start..end)` returning `Option`, never with `[start..end]`.
//! Concretely:
//!
//! 1. `header_len` comes from the length-field width plus an optional 4-byte key,
//!    so it is at most 14 and is already known to be buffered.
//! 2. `payload_len` was bounded by [`crate::websocket::limits::check_payload`]
//!    before this point, so `header_len + payload_len` cannot overflow `usize`
//!    on any supported target (the sum is under 2^25 + 14).
//! 3. If the buffer is shorter than that sum, `get` yields `None` and the result
//!    is `Incomplete` — a short read, not a panic and not a malformed frame.
//!
//! No allocation is sized from the declared length either; the payload `Vec` is
//! built from an already-materialized slice, so a peer cannot make the process
//! reserve 16 MiB by *claiming* 16 MiB.

use crate::websocket::error::ProtocolError;
use crate::websocket::frame::{DecodeOutcome, Frame};
use crate::websocket::header::FrameHeader;
use crate::websocket::mask;
use crate::websocket::role::Role;
use crate::websocket::validate;

/// Decode one frame from the front of `bytes`.
///
/// # Arguments
///
/// * `bytes` — Read buffer positioned at a frame boundary.
/// * `role` — Sender of the frame; enforces the masking direction.
///
/// # Returns
///
/// [`DecodeOutcome::Frame`] with the bytes consumed, or
/// [`DecodeOutcome::Incomplete`] with nothing consumed.
///
/// # Errors
///
/// Any [`ProtocolError`] from header validation, or from payload validation of a
/// close frame (its code and reason are checked here, since a bad close code must
/// not be handed to the application).
///
/// # Panics
///
/// Never; see the module documentation for the index-safety argument.
pub fn decode(bytes: &[u8], role: Role) -> Result<DecodeOutcome, ProtocolError> {
    let Some(header) = FrameHeader::parse(bytes, role)? else {
        return Ok(DecodeOutcome::Incomplete);
    };
    let end = header.header_len + header.payload_len;
    let Some(body) = bytes.get(header.header_len..end) else {
        return Ok(DecodeOutcome::Incomplete);
    };
    let mut payload = body.to_vec();
    if let Some(key) = header.mask {
        mask::apply(&mut payload, key);
    }
    validate::payload(header.opcode, header.fin, &payload)?;
    Ok(DecodeOutcome::Frame {
        frame: Frame {
            fin: header.fin,
            opcode: header.opcode,
            payload,
        },
        consumed: end,
    })
}
