//! Definite-length octet decoding with DER's minimality rules enforced.
//!
//! DER permits exactly one encoding of any length: the short form for values
//! below 128, and otherwise the long form with no leading zero octets.
//! Indefinite length (`0x80`) and the reserved `0xFF` are rejected outright.
//! Rejecting the alternatives matters for signature security: if `0x02 0x01
//! 0x05` and `0x02 0x81 0x01 0x05` both parsed, one logical key would have two
//! byte encodings and therefore two distinct valid signatures.

use super::error::Error;

/// Decode the length field starting at `pos`.
///
/// # Arguments
///
/// * `input` — the buffer being parsed.
/// * `pos` — index of the first length octet within `input`.
/// * `base` — offset of `input[0]` within the original document, used only to
///   report absolute error offsets.
///
/// # Returns
///
/// `(length, next)` where `length` is the content-octet count and `next` is the
/// index just past the length field.
///
/// # Errors
///
/// [`Error::UnexpectedEnd`] if the field is truncated,
/// [`Error::IndefiniteLength`], [`Error::ReservedLength`],
/// [`Error::NonMinimalLength`], or [`Error::LengthTooLarge`] when the field is
/// wider than a `usize`.
///
/// # Panics
///
/// Never. Every read goes through `slice::get`, and the accumulator shifts by a
/// constant 8 bits at most `size_of::<usize>()` times, so no value is truncated
/// and no shift is out of range.
pub(super) fn decode(input: &[u8], pos: usize, base: usize) -> Result<(usize, usize), Error> {
    let at = base.saturating_add(pos);
    let first = *input.get(pos).ok_or(Error::UnexpectedEnd { offset: at })?;
    if first < 0x80 {
        return Ok((usize::from(first), pos.saturating_add(1)));
    }
    if first == 0x80 {
        return Err(Error::IndefiniteLength { offset: at });
    }
    if first == 0xff {
        return Err(Error::ReservedLength { offset: at });
    }
    super::length_long::decode(input, pos, base, usize::from(first & 0x7f))
}
