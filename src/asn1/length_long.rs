//! Long-form (multi-byte) definite length decoding.
//!
//! The long form is `0x80 | n` followed by `n` big-endian length octets. DER
//! adds two minimality rules that this module enforces:
//!
//! 1. the first of the `n` octets must be non-zero (no leading zero padding);
//! 2. the encoded value must be at least 128, since anything smaller has a
//!    short form.
//!
//! A `0x84 0xFF 0xFF 0xFF 0xFF` header claiming 4 GiB is decoded to the number
//! `4294967295` here and then rejected by the caller's bounds check against the
//! remaining input, so nothing is ever allocated on its behalf.

use super::error::Error;

/// Decode `count` big-endian length octets following the long-form marker.
///
/// # Arguments
///
/// * `input` — the buffer being parsed.
/// * `pos` — index of the long-form marker octet.
/// * `base` — offset of `input[0]` within the original document.
/// * `count` — number of subsequent length octets, always `1..=126` here.
///
/// # Returns
///
/// `(length, next)` where `next` is the index just past the final length octet.
///
/// # Errors
///
/// [`Error::LengthTooLarge`] when `count` exceeds the width of a `usize`,
/// [`Error::UnexpectedEnd`] when the octets are truncated, and
/// [`Error::NonMinimalLength`] for leading zeros or an over-wide encoding.
///
/// # Panics
///
/// Never: the octet range is taken with `slice::get`, and `count` is bounded to
/// `size_of::<usize>()` before any shifting, so `value << 8` cannot overflow.
pub(super) fn decode(
    input: &[u8],
    pos: usize,
    base: usize,
    count: usize,
) -> Result<(usize, usize), Error> {
    let at = base.saturating_add(pos);
    if count > std::mem::size_of::<usize>() {
        return Err(Error::LengthTooLarge {
            offset: at,
            bytes: count,
        });
    }
    let start = pos.saturating_add(1);
    let end = start.saturating_add(count);
    let octets = input
        .get(start..end)
        .ok_or(Error::UnexpectedEnd { offset: at })?;
    if octets.first() == Some(&0) {
        return Err(Error::NonMinimalLength { offset: at });
    }
    let mut value: usize = 0;
    for octet in octets {
        value = (value << 8) | usize::from(*octet);
    }
    if value < 0x80 {
        return Err(Error::NonMinimalLength { offset: at });
    }
    Ok((value, end))
}
