//! Decoding the variable-width payload length field (RFC 6455 §5.2).
//!
//! The length is one of three forms selected by the low seven bits of byte 1.
//! Two rules are enforced here that a naive reader skips:
//!
//! * The 64-bit form's most significant bit **must** be zero. A peer that sets
//!   it is rejected rather than having the value truncated.
//! * The encoding must be **minimal**. `126 0x00 0x05` encodes 5 in the 16-bit
//!   form and is refused, because tolerating redundant encodings gives an
//!   attacker two byte sequences for one frame and defeats byte-exact
//!   deduplication or logging upstream.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::header_len::decode;
//!
//! // 7-bit form: length 5 ends at offset 2.
//! assert_eq!(decode(&[0x81, 0x05]).unwrap(), Some((5, 2)));
//! // Truncated 16-bit form: not an error, just not here yet.
//! assert_eq!(decode(&[0x81, 0x7e, 0x01]).unwrap(), None);
//! ```

use crate::websocket::error::ProtocolError;

/// Read the payload length and report where the length field ends.
///
/// # Arguments
///
/// * `bytes` — Buffer starting at the frame's first byte.
///
/// # Returns
///
/// `Ok(Some((len, next)))` where `next` is the offset just past the length
/// field, or `Ok(None)` when `bytes` does not yet hold the whole field.
///
/// # Errors
///
/// [`ProtocolError::LengthMsbSet`] or [`ProtocolError::NonMinimalLength`].
///
/// # Panics
///
/// Never. Every read goes through `get`, so a short buffer yields `None`
/// instead of indexing out of range.
pub fn decode(bytes: &[u8]) -> Result<Option<(u64, usize)>, ProtocolError> {
    let Some(indicator) = bytes.get(1).map(|byte| byte & 0x7f) else {
        return Ok(None);
    };
    match indicator {
        0..=125 => Ok(Some((u64::from(indicator), 2))),
        126 => wide(bytes, 2, 4, 126),
        _ => wide(bytes, 2, 10, 65_536),
    }
}

/// Read a big-endian extended length from `bytes[start..end]`.
///
/// # Arguments
///
/// * `bytes` — Buffer starting at the frame's first byte.
/// * `start` — First byte of the extended field (always 2).
/// * `end` — One past its last byte: 4 for the 16-bit form, 10 for the 64-bit.
/// * `floor` — Smallest value this form is allowed to encode.
///
/// # Returns
///
/// `Ok(Some((len, end)))`, or `Ok(None)` when the field is not fully buffered.
///
/// # Errors
///
/// [`ProtocolError::LengthMsbSet`] when bit 63 is set in the 64-bit form, and
/// [`ProtocolError::NonMinimalLength`] when `len < floor`.
fn wide(
    bytes: &[u8],
    start: usize,
    end: usize,
    floor: u64,
) -> Result<Option<(u64, usize)>, ProtocolError> {
    let Some(field) = bytes.get(start..end) else {
        return Ok(None);
    };
    let mut raw = 0u64;
    for byte in field {
        raw = (raw << 8) | u64::from(*byte);
    }
    if end == 10 && raw & 0x8000_0000_0000_0000 != 0 {
        return Err(ProtocolError::LengthMsbSet { raw });
    }
    if raw < floor {
        return Err(ProtocolError::NonMinimalLength { len: raw });
    }
    Ok(Some((raw, end)))
}
