//! Identifier-plus-length header parsing: the single place a content slice is
//! produced.
//!
//! Because this is the only function that slices content out of the buffer, one
//! bounds check here covers the whole decoder. `input.get(start..end)` returns
//! `None` rather than panicking whenever the declared length runs past the end
//! of the input, and that `None` becomes [`Error::LengthExceedsInput`]. Nothing
//! is allocated on the strength of a declared length, so a header claiming 4 GiB
//! costs one failed range check.

use super::{error::Error, length, tag, tlv::Tlv};

/// Parse one TLV header and slice out its content octets.
///
/// # Arguments
///
/// * `input` — the buffer being parsed.
/// * `pos` — index of the identifier octet within `input`.
/// * `base` — offset of `input[0]` within the original document.
///
/// # Returns
///
/// `(tlv, next)` where `next` is the index in `input` just past the value.
///
/// # Errors
///
/// [`Error::UnexpectedEnd`] for a missing identifier octet,
/// [`Error::HighTagNumber`] for multi-byte tags, any length error from
/// [`length::decode`], and [`Error::LengthExceedsInput`] when the declared
/// length exceeds the bytes actually present.
///
/// # Panics
///
/// Never. The identifier octet is read with `slice::get`, the content range is
/// taken with `slice::get(..)`, and every offset uses `saturating_add`.
pub(super) fn parse(input: &[u8], pos: usize, base: usize) -> Result<(Tlv<'_>, usize), Error> {
    let at = base.saturating_add(pos);
    let identifier = *input.get(pos).ok_or(Error::UnexpectedEnd { offset: at })?;
    if tag::is_high_tag_number(identifier) {
        return Err(Error::HighTagNumber { offset: at });
    }
    let (length, start) = length::decode(input, pos.saturating_add(1), base)?;
    let end = start.saturating_add(length);
    let content = input.get(start..end).ok_or(Error::LengthExceedsInput {
        offset: base.saturating_add(start),
        length,
        available: input.len().saturating_sub(start),
    })?;
    let tlv = Tlv {
        tag: identifier,
        offset: at,
        content_offset: base.saturating_add(start),
        content,
    };
    Ok((tlv, end))
}
