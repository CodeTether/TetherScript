//! # Splitting and chunking decimal digit strings
//!
//! The purely lexical half of `numeric` encoding: peel the sign, split on the decimal
//! point, validate that everything left is an ASCII digit, and chunk each half into
//! 4-digit base-10000 groups.
//!
//! **The padding side differs per half, and that is the easiest thing to get wrong.**
//! Integer digits are left-padded — `12345` becomes `0001|2345`, because the *last*
//! integer group is the units. Fraction digits are right-padded — `5` becomes
//! `5000`, because the *first* fraction group is the ten-thousandths. Padding either
//! half on the wrong side changes the value by a factor of 10, 100, or 1000 while
//! still producing a perfectly well-formed `numeric`.
//!
//! Chunking into 4-digit groups lives in the sibling
//! `binary_encode_numeric_chunk.rs`; this file owns sign peeling, the decimal-point
//! split, and digit validation.

use super::super::super::error::DecodeError;
use super::groups::bad;

/// Peel an optional leading `-` or `+`.
///
/// # Arguments
///
/// * `text` — the trimmed decimal literal.
///
/// # Returns
///
/// `(negative, remaining_digits)`.
pub(super) fn sign(text: &str) -> (bool, &str) {
    if let Some(rest) = text.strip_prefix('-') {
        return (true, rest);
    }
    (false, text.strip_prefix('+').unwrap_or(text))
}

/// Split on the decimal point into integer and fraction digit strings.
///
/// # Arguments
///
/// * `digits` — the literal with its sign already removed.
///
/// # Returns
///
/// `(integer_digits, fraction_digits)`; either may be empty, and an empty integer
/// half is treated as `0` by the padding step.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for more than one `.`, for a non-digit character, or
/// when both halves are empty.
pub(super) fn halves(digits: &str) -> Result<(&str, &str), DecodeError> {
    let mut parts = digits.split('.');
    let integer = parts.next().unwrap_or("");
    let fraction = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return Err(bad(format!("{digits:?} has more than one decimal point")));
    }
    if integer.is_empty() && fraction.is_empty() {
        return Err(bad(format!("{digits:?} contains no digits")));
    }
    for half in [integer, fraction] {
        if let Some(bad_char) = half.chars().find(|c| !c.is_ascii_digit()) {
            return Err(bad(format!(
                "{digits:?} contains {bad_char:?}, which is not a decimal digit"
            )));
        }
    }
    Ok((integer, fraction))
}

/// Chunk integer digits into groups, **left**-padding to a multiple of 4.
///
/// # Arguments
///
/// * `digits` — validated ASCII integer digits, possibly empty.
///
/// # Returns
///
/// Groups most significant first; `[0]` for an empty or all-zero input, so the weight
/// calculation always has an integer group to count.
///
/// # Errors
///
/// [`DecodeError::BadValue`] if a 4-digit chunk fails to parse, which the digit
/// validation in [`halves`] should already have prevented.
pub(super) fn chunk_left_padded(digits: &str) -> Result<Vec<u16>, DecodeError> {
    let source = if digits.is_empty() { "0" } else { digits };
    let pad = (4 - source.len() % 4) % 4;
    super::chunk::parse_groups(&format!("{}{source}", "0".repeat(pad)))
}

/// Chunk fraction digits into groups, **right**-padding to a multiple of 4.
///
/// # Arguments
///
/// * `digits` — validated ASCII fraction digits, possibly empty.
///
/// # Returns
///
/// Groups most significant first, or an empty vector for no fraction.
///
/// # Errors
///
/// As [`chunk_left_padded`].
pub(super) fn chunk_right_padded(digits: &str) -> Result<Vec<u16>, DecodeError> {
    if digits.is_empty() {
        return Ok(Vec::new());
    }
    let pad = (4 - digits.len() % 4) % 4;
    super::chunk::parse_groups(&format!("{digits}{}", "0".repeat(pad)))
}
