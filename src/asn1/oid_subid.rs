//! Base-128 subidentifier decoding for OBJECT IDENTIFIERs.
//!
//! Each subidentifier is a run of octets whose top bit marks continuation. DER
//! forbids a leading `0x80`, since that would be a redundant zero group, and
//! forbids the content ending while the continuation bit is still set.

use super::{error::Error, tag};

/// Decode one subidentifier starting at `pos`.
///
/// # Arguments
///
/// * `content` — the OID's content octets.
/// * `pos` — index of the subidentifier's first octet.
/// * `offset` — absolute offset of the OID's tag, for error reporting.
///
/// # Returns
///
/// `(value, next)` where `next` indexes the octet after this subidentifier.
///
/// # Errors
///
/// [`Error::MalformedValue`] for a non-minimal `0x80` lead octet, a truncated
/// subidentifier, or a value wider than 63 significant bits.
///
/// # Panics
///
/// Never. The loop is bounded by `content.len()`, octets are read with
/// `slice::get`, and the shift is guarded by an explicit `u64` range check, so
/// `value << 7` cannot overflow.
pub(super) fn next(content: &[u8], pos: usize, offset: usize) -> Result<(u64, usize), Error> {
    let bad = |reason| Error::MalformedValue {
        offset,
        tag: tag::OBJECT_IDENTIFIER,
        reason,
    };
    if content.get(pos) == Some(&0x80) {
        return Err(bad("OID subidentifier has a non-minimal 0x80 lead octet"));
    }
    let mut value: u64 = 0;
    let mut index = pos;
    while let Some(octet) = content.get(index).copied() {
        if value > (u64::MAX >> 7) {
            return Err(bad("OID subidentifier is too large for 64 bits"));
        }
        value = (value << 7) | u64::from(octet & 0x7f);
        index = index.saturating_add(1);
        if octet & 0x80 == 0 {
            return Ok((value, index));
        }
    }
    Err(bad("OID subidentifier ended with the continuation bit set"))
}
