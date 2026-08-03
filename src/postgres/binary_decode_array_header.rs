//! # The binary array header
//!
//! `ndim`, `has_null`, `element_oid`, then one `(length, lower_bound)` pair per
//! dimension — all **big-endian** `int32`. Parsing is separated from element
//! decoding so every rejection rule sits in one reviewable place.
//!
//! Three checks, each guarding an untrusted number:
//!
//! 1. **`ndim` must be 0 or 1.** Higher counts are rejected by name rather than
//!    flattened, because flattening discards shape irrecoverably. `ndim == 0` is the
//!    empty array and carries *no* dimension block, so the loop must not run.
//! 2. **The element OID must match the column.** The array's own OID told the caller
//!    which element decoder to use; if the header disagrees, one of the two is wrong
//!    and decoding either way would produce a plausible wrong value.
//! 3. **The dimension length must be non-negative.** It becomes a loop bound and a
//!    `Vec` capacity, so a negative value cast to `usize` would be catastrophic.
//!
//! `has_null` is read and deliberately ignored: the per-element length of -1 is
//! authoritative, and trusting an advisory flag over the actual framing is how a
//! decoder ends up out of step with the bytes in front of it.

use super::super::super::error::DecodeError;
use super::super::super::read::Reader;

/// Read and validate the header, returning the element count to read.
///
/// # Arguments
///
/// * `reader` — positioned at the start of the field body.
/// * `expected_element` — element OID derived from the column's array OID.
///
/// # Returns
///
/// The number of elements that follow: 0 for an empty array, otherwise the single
/// dimension's length.
///
/// # Errors
///
/// [`DecodeError::UnsupportedDimensions`] for `ndim` outside 0..=1,
/// [`DecodeError::BadValue`] for an element-OID mismatch or a negative dimension
/// length, and [`DecodeError::Truncated`] on a short header.
pub(super) fn parse(reader: &mut Reader<'_>, expected_element: u32) -> Result<usize, DecodeError> {
    let ndim = reader.i32("array ndim")?;
    let _has_null = reader.i32("array has_null")?; // advisory; lengths are truth
    let element_oid = reader.i32("array element oid")? as u32;
    if !(0..=1).contains(&ndim) {
        return Err(DecodeError::UnsupportedDimensions { ndim });
    }
    if element_oid != expected_element {
        return Err(DecodeError::BadValue {
            what: "array",
            detail: format!(
                "header element OID {element_oid} contradicts the column's {expected_element}"
            ),
        });
    }
    if ndim == 0 {
        // An empty array carries no dimension block at all.
        return Ok(0);
    }
    let length = reader.i32("array dimension length")?;
    let _lower_bound = reader.i32("array lower bound")?; // normally 1; not needed
    if length < 0 {
        return Err(DecodeError::BadValue {
            what: "array",
            detail: format!("negative dimension length {length}"),
        });
    }
    Ok(length as usize)
}
