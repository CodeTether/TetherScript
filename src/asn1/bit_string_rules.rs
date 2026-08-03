//! The BIT STRING content rules: unused-bit count validation and padding checks.

use super::{bit_string::BitString, error::Error, tag};

/// Split and validate BIT STRING content octets.
///
/// # Arguments
///
/// * `content` — the full content octets, including the unused-bits prefix.
/// * `offset` — absolute offset of the BIT STRING's tag.
///
/// # Returns
///
/// A [`BitString`] borrowing the value octets.
///
/// # Errors
///
/// [`Error::MalformedValue`] when the content is empty, the count exceeds 7,
/// a non-zero count accompanies no value octets, or the declared unused bits in
/// the final octet are not zero.
///
/// # Panics
///
/// Never; the slice is destructured with patterns and `last`, never indexed.
pub(super) fn decode(content: &[u8], offset: usize) -> Result<BitString<'_>, Error> {
    let bad = |reason| Error::MalformedValue {
        offset,
        tag: tag::BIT_STRING,
        reason,
    };
    let [unused_bits, bytes @ ..] = content else {
        return Err(bad("BIT STRING must have an unused-bits octet"));
    };
    let unused_bits = *unused_bits;
    if unused_bits > 7 {
        return Err(bad("BIT STRING unused-bit count exceeds 7"));
    }
    if bytes.is_empty() && unused_bits != 0 {
        return Err(bad("empty BIT STRING must declare zero unused bits"));
    }
    if let Some(last) = bytes.last() {
        if unused_bits > 0 && last & ((1u8 << unused_bits) - 1) != 0 {
            return Err(bad("BIT STRING unused bits must be zero"));
        }
    }
    Ok(BitString { unused_bits, bytes })
}
