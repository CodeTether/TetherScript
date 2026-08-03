//! Header parsing and validation order for [`FrameHeader`].
//!
//! Split from `header.rs` so the public shape and the byte-level rules stay in
//! separate files. The *order* of checks below is load-bearing: cheap, purely
//! structural rejections (RSV, opcode, masking direction, control-frame rules)
//! run before the declared length is ever converted into a `usize` or used to
//! size anything, so a hostile header costs a peer nothing but a rejection.

use crate::websocket::error::ProtocolError;
use crate::websocket::header::FrameHeader;
use crate::websocket::header_flags::{decode_flags, take_mask};
use crate::websocket::header_len;
use crate::websocket::limits::{self, MAX_CONTROL_PAYLOAD_LEN};
use crate::websocket::role::Role;

/// Parse and fully validate a frame header.
///
/// # Arguments
///
/// * `bytes` — Buffer starting at the frame's first byte.
/// * `role` — Sender of the frame; decides the masking requirement.
///
/// # Returns
///
/// `Ok(Some(header))`, or `Ok(None)` if the header is not yet fully buffered.
///
/// # Errors
///
/// Any framing [`ProtocolError`]; see [`FrameHeader::parse`].
pub(super) fn parse_validated(
    bytes: &[u8],
    role: Role,
) -> Result<Option<FrameHeader>, ProtocolError> {
    let Some((fin, opcode, masked)) = decode_flags(bytes)? else {
        return Ok(None);
    };
    if masked != role.requires_mask() {
        return Err(match role {
            Role::Client => ProtocolError::UnmaskedClientFrame,
            Role::Server => ProtocolError::MaskedServerFrame,
        });
    }
    let Some((declared, after_len)) = header_len::decode(bytes)? else {
        return Ok(None);
    };
    if opcode.is_control() {
        if !fin {
            return Err(ProtocolError::FragmentedControlFrame);
        }
        if declared > MAX_CONTROL_PAYLOAD_LEN as u64 {
            return Err(ProtocolError::ControlPayloadTooLarge { len: declared });
        }
    }
    limits::check_payload(declared)?;
    let Some((mask, header_len)) = take_mask(bytes, after_len, masked) else {
        return Ok(None);
    };
    Ok(Some(FrameHeader {
        fin,
        opcode,
        mask,
        payload_len: declared as usize,
        header_len,
    }))
}
