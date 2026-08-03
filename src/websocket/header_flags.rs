//! The bit flags of frame byte 0 and the optional masking key.
//!
//! RSV1/RSV2/RSV3 are rejected when set rather than ignored. They only carry
//! meaning under a negotiated extension (`permessage-deflate`, for example), and
//! this codec negotiates none — so a set RSV bit means the peer believes
//! something about the payload that this decoder does not, and continuing would
//! mean interpreting bytes under the wrong contract.

use crate::websocket::error::ProtocolError;
use crate::websocket::opcode::Opcode;

/// Decode FIN, the RSV bits, the opcode, and MASK.
///
/// # Arguments
///
/// * `bytes` — Buffer starting at the frame's first byte.
///
/// # Returns
///
/// `Ok(Some((fin, opcode, masked)))`, or `Ok(None)` when fewer than two bytes
/// are buffered.
///
/// # Errors
///
/// [`ProtocolError::ReservedBitSet`] or [`ProtocolError::ReservedOpcode`].
///
/// # Panics
///
/// Never. Both bytes are read through `first`/`get` and copied out, so a short
/// buffer yields `None` rather than indexing out of range.
pub(super) fn decode_flags(bytes: &[u8]) -> Result<Option<(bool, Opcode, bool)>, ProtocolError> {
    let (Some(first), Some(second)) = (bytes.first().copied(), bytes.get(1).copied()) else {
        return Ok(None);
    };
    let rsv = (first >> 4) & 0x07;
    if rsv != 0 {
        return Err(ProtocolError::ReservedBitSet { rsv });
    }
    let opcode =
        Opcode::from_bits(first).ok_or(ProtocolError::ReservedOpcode { bits: first & 0x0f })?;
    Ok(Some((first & 0x80 != 0, opcode, second & 0x80 != 0)))
}

/// Read the four-byte masking key when MASK is set.
///
/// # Arguments
///
/// * `bytes` — Buffer starting at the frame's first byte.
/// * `after_len` — Offset just past the payload-length field.
/// * `masked` — Whether MASK was set in byte 1.
///
/// # Returns
///
/// `Some((mask, header_len))`, or `None` when the key is not fully buffered.
///
/// # Panics
///
/// Never. The slice is taken with `get`, and `try_into` on a 4-byte slice is
/// infallible, so the `unwrap_or` fallback is dead but keeps the path panic-free.
pub(super) fn take_mask(
    bytes: &[u8],
    after_len: usize,
    masked: bool,
) -> Option<(Option<[u8; 4]>, usize)> {
    if !masked {
        return Some((None, after_len));
    }
    let end = after_len + 4;
    let field = bytes.get(after_len..end)?;
    let key: [u8; 4] = field.try_into().unwrap_or([0; 4]);
    Some((Some(key), end))
}
