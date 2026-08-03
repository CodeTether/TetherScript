//! # Decimal string to base-10000 digit groups
//!
//! Converts `-12345.6789` into `weight = 1`, `dscale = 4`,
//! `groups = [1, 2345, 6789]` — the exact representation the wire expects — using
//! only digit-string manipulation and 4-digit integer parses. **No bigint, no
//! floating point.** That is what makes it exact for an arbitrarily long decimal,
//! which is the whole reason to bind `numeric` rather than `float8`.
//!
//! ## The algorithm
//!
//! 1. Split on `.` into integer and fraction digits; `dscale` is the fraction length
//!    *as written*, so a trailing zero the caller typed is preserved — `0.50` stays
//!    two decimal places, because a price displayed as `0.5` is wrong.
//! 2. Left-pad the integer digits to a multiple of 4, right-pad the fraction to a
//!    multiple of 4. Padding on the correct side per half is the crux: left-padding
//!    the fraction would turn `.5` into `.0005`.
//! 3. Chunk both into 4-digit groups; `weight` is `integer_group_count - 1`.
//! 4. Strip leading zero groups, decrementing `weight` each time, and strip trailing
//!    zero groups. PostgreSQL never transmits them, and the decoder's implicit-zero
//!    rule restores them.
//!
//! An all-zero value ends with no groups at all and `weight = 0`, matching how the
//! server encodes `0`.

//! Splitting and chunking live in the sibling `binary_encode_numeric_split.rs`.

use super::super::super::error::DecodeError;
use super::split;

/// A decimal decomposed into wire form.
pub(super) struct Parsed {
    /// Whether a `-` sign was present.
    pub(super) negative: bool,
    /// Base-10000 exponent of the first group.
    pub(super) weight: i16,
    /// Display scale: fraction digits exactly as written.
    pub(super) dscale: i16,
    /// The digit groups, leading and trailing zero groups removed.
    pub(super) groups: Vec<u16>,
}

/// Parse a decimal literal into its base-10000 wire representation.
///
/// # Arguments
///
/// * `text` — a decimal literal: optional `-` or `+`, digits, optional `.` and more
///   digits. No exponent notation.
///
/// # Returns
///
/// The [`Parsed`] decomposition, ready to serialise.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-digit character, more than one `.`, an empty
/// digit string, or a scale or group count too large for the `i16` header fields.
pub(super) fn parse(text: &str) -> Result<Parsed, DecodeError> {
    let (negative, digits) = split::sign(text);
    let (int_digits, frac_digits) = split::halves(digits)?;
    let dscale = i16::try_from(frac_digits.len())
        .map_err(|_| bad(format!("scale {} exceeds the i16 header field", frac_digits.len())))?;
    let int_groups = split::chunk_left_padded(int_digits)?;
    let frac_groups = split::chunk_right_padded(frac_digits)?;
    let mut weight = int_groups.len() as i64 - 1;
    let mut groups = int_groups;
    groups.extend(frac_groups);
    // Leading zero groups are never transmitted; each one lowers the weight.
    while groups.first() == Some(&0) {
        groups.remove(0);
        weight -= 1;
    }
    while groups.last() == Some(&0) {
        groups.pop();
    }
    if groups.is_empty() {
        weight = 0; // canonical zero: no digits, weight 0
    }
    let weight = i16::try_from(weight)
        .map_err(|_| bad(format!("weight {weight} exceeds the i16 header field")))?;
    Ok(Parsed {
        negative,
        weight,
        dscale,
        groups,
    })
}

/// Build a named `numeric` encoding error.
pub(super) fn bad(detail: String) -> DecodeError {
    DecodeError::BadValue {
        what: "numeric",
        detail,
    }
}
