//! # Reassembling `numeric` digit groups into an exact decimal string
//!
//! The value is `sign * Σ groups[i] * 10000^(weight - i)`. So the group holding
//! decimal exponent `e` sits at index `i = weight - e`: exponents `weight..=0` form
//! the integer part and `-1, -2, …` the fraction. Any index outside
//! `0..ndigits` is an **implicit zero**, which is how a value like `0.00000001`
//! with a large negative weight, or a `dscale` wider than the groups sent, comes
//! out right rather than short.
//!
//! No floating point appears anywhere in this file. Every digit is produced by
//! integer division on a `u16` group and by string concatenation, so the result is
//! exact for any magnitude and any scale — which is the entire point of `numeric`.
//!
//! Rendering lives in the sibling `binary_decode_numeric_render.rs` so each file owns
//! one concern: this one reads and validates groups, that one formats them.

use super::super::super::error::DecodeError;
use super::super::super::read::Reader;
use super::header::Header;

/// Largest legal base-10000 digit group.
pub(super) const MAX_GROUP: i16 = 9_999;

/// Read `count` big-endian base-10000 digit groups.
///
/// # Arguments
///
/// * `reader` — positioned just past the header.
/// * `count` — validated `ndigits` from the header.
///
/// # Returns
///
/// The groups in wire order, most significant first.
///
/// # Errors
///
/// [`DecodeError::Truncated`] when a group is short, and [`DecodeError::BadValue`]
/// for a group outside `0..=9999` — digits are always non-negative, since the sign
/// lives in the header, so a negative group means the frame is not a `numeric`.
pub(super) fn read_groups(reader: &mut Reader<'_>, count: usize) -> Result<Vec<u16>, DecodeError> {
    let mut groups = Vec::with_capacity(count.min(1_024));
    for _ in 0..count {
        let group = reader.i16("numeric digit group")?;
        if !(0..=MAX_GROUP).contains(&group) {
            return Err(DecodeError::BadValue {
                what: "numeric",
                detail: format!("digit group {group} outside 0..={MAX_GROUP}"),
            });
        }
        groups.push(group as u16);
    }
    Ok(groups)
}

/// The group holding decimal exponent `e`, or 0 when it was not transmitted.
///
/// # Arguments
///
/// * `header` — supplies `weight`.
/// * `groups` — the transmitted groups.
/// * `exponent` — the base-10000 exponent wanted.
///
/// # Returns
///
/// `groups[weight - exponent]`, or 0 when that index is outside the slice — the
/// implicit-zero rule that makes leading and trailing gaps decode correctly.
pub(super) fn group_at(header: &Header, groups: &[u16], exponent: i32) -> u16 {
    let index = header.weight as i32 - exponent;
    if index < 0 {
        return 0;
    }
    groups.get(index as usize).copied().unwrap_or(0)
}

/// Concatenate a run of exponents as four zero-padded decimal digits each.
///
/// # Arguments
///
/// * `header` — supplies `weight` for the index mapping.
/// * `groups` — the transmitted groups.
/// * `exponents` — the base-10000 exponents to emit, in output order.
///
/// # Returns
///
/// `4 * n` decimal digits for `n` exponents. Callers trim or truncate: the integer
/// part strips leading zeros, the fraction part truncates to `dscale`. Padding
/// every group uniformly here is what keeps an interior group like `7` rendering as
/// `0007` rather than collapsing into the digit above it.
pub(super) fn render_group_run(
    header: &Header,
    groups: &[u16],
    exponents: impl Iterator<Item = i32>,
) -> String {
    let mut out = String::new();
    for exponent in exponents {
        let group = group_at(header, groups, exponent);
        out.push_str(&format!("{group:04}"));
    }
    out
}
