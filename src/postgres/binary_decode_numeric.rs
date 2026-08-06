//! # Exact `numeric` decoding — never through `f64`
//!
//! `numeric` exists precisely so a decimal is *not* binary floating point. A money
//! column decoded through a `double` is a correctness bug: `0.1` is unrepresentable
//! in binary, so `19.99` becomes `19.989999999999998` and a sum of a thousand rows
//! drifts by cents. This decoder therefore performs **no floating-point arithmetic
//! at all** — it reassembles the decimal digits as text and hands back a
//! [`Value::Str`] holding the exact value. A caller that wants a float can convert
//! deliberately; a caller that wants exactness keeps it.
//!
//! ## Wire layout
//!
//! All four header words and every digit group are **big-endian**:
//!
//! ```text
//! int16  ndigits   number of base-10000 digit groups that follow
//! int16  weight    base-10000 exponent of the FIRST group (0 => units)
//! uint16 sign      0x0000 pos, 0x4000 neg, 0xC000 NaN, 0xD000 +Inf, 0xF000 -Inf
//! int16  dscale    display scale: decimal digits to show after the point
//! int16  digits[ndigits]   each 0..=9999
//! ```
//!
//! The value is `sign * Σ digits[i] * 10000^(weight - i)`. Note that `weight` is in
//! units of **10000**, not 10: a weight of 1 means the first group is thousands to
//! ten-thousands. Treating it as a power of ten is off by a factor of 1000.
//!
//! ## Cases that are easy to get wrong
//!
//! - **Zero** is `ndigits = 0`, so the digit loop never runs and the integer part
//!   must still print `0`.
//! - **NaN** has `ndigits = 0` too, and is distinguished only by the sign word. It
//!   is rendered `NaN` — the string PostgreSQL itself uses and accepts back.
//! - **Negative** is a sign word, not a two's-complement digit; digits are always
//!   non-negative.
//! - **High scale**: `dscale` can exceed `ndigits * 4`, in which case the missing
//!   low-order groups are implicit zeros, and it can exceed the digits present at
//!   the top too.

use std::rc::Rc;

use crate::value::Value;

use super::super::error::DecodeError;
use super::super::read::Reader;

#[path = "binary_decode_numeric_digits.rs"]
mod digits;
#[path = "binary_decode_numeric_header.rs"]
mod header;
#[path = "binary_decode_numeric_render.rs"]
mod render;
#[path = "binary_decode_numeric_sign.rs"]
mod sign;

/// Decode a binary `numeric` into its exact decimal string.
///
/// # Arguments
///
/// * `body` — the field body: the 8-byte header then `ndigits` 2-byte groups.
///
/// # Returns
///
/// [`Value::Str`] holding the exact decimal, or `NaN` / `Infinity` / `-Infinity`.
///
/// # Errors
///
/// - [`DecodeError::Truncated`] when the header or a digit group is short.
/// - [`DecodeError::Overlong`] when more groups were sent than `ndigits` claims.
/// - [`DecodeError::BadNumericSign`] for an unrecognised sign word.
/// - [`DecodeError::BadValue`] for a negative `ndigits`/`dscale`, an out-of-range
///   digit group, or a `dscale` beyond PostgreSQL's 16383 maximum — each of which
///   would otherwise drive an unbounded or nonsensical allocation.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{decode_field, oid};
/// use tetherscript::value::Value;
///
/// // 12345.6789 => ndigits 3, weight 1, sign 0, dscale 4, groups [1, 2345, 6789]
/// let body = [0, 3, 0, 1, 0, 0, 0, 4, 0, 1, 9, 41, 26, 133];
/// let decoded = decode_field(oid::NUMERIC, &body).unwrap();
/// assert_eq!(decoded, Value::Str(std::rc::Rc::new("12345.6789".into())));
/// ```
pub(super) fn decode(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let header = header::parse(&mut reader)?;
    if let Some(special) = header.special() {
        // NaN and the infinities carry no digits worth reading.
        return Ok(Value::Str(Rc::new(special.to_string())));
    }
    let groups = digits::read_groups(&mut reader, header.ndigits)?;
    reader.finish("numeric")?;
    Ok(Value::Str(Rc::new(render::render(&header, &groups))))
}

/// Decode a binary `numeric` straight to its exact decimal string.
///
/// Convenience wrapper for hosts that want the text without unwrapping a
/// [`Value`]. Identical semantics and identical errors to the column decoder.
///
/// # Arguments
///
/// * `body` — the binary `numeric` field body.
///
/// # Returns
///
/// The exact decimal string.
///
/// # Errors
///
/// As `decode`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::numeric_to_string;
///
/// // Zero: no digit groups at all.
/// assert_eq!(numeric_to_string(&[0, 0, 0, 0, 0, 0, 0, 0]).unwrap(), "0");
/// // NaN is the sign word 0xC000.
/// assert_eq!(numeric_to_string(&[0, 0, 0, 0, 0xC0, 0, 0, 0]).unwrap(), "NaN");
/// ```
pub fn numeric_to_string(body: &[u8]) -> Result<String, DecodeError> {
    match decode(body)? {
        Value::Str(text) => Ok(text.as_ref().clone()),
        other => Err(DecodeError::BadValue {
            what: "numeric",
            detail: format!("decoder produced a {} value", other.type_name()),
        }),
    }
}
