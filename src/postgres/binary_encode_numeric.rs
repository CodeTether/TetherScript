//! # Exact `numeric` parameter encoding
//!
//! The inverse of `binary_decode_numeric.rs`, and it carries the same prohibition:
//! **no floating point anywhere.** A caller binding `19.99` to a money column must
//! get `19.99` on the wire, not `19.989999999999998`. So the input is a *decimal
//! string* and it is converted digit by digit into base-10000 groups.
//!
//! A [`Value::Int`] is also accepted, since an integer is exactly representable. A
//! [`Value::Float`] is **rejected**: the float has already lost the exactness
//! `numeric` exists to preserve, so accepting it would launder a precision bug into a
//! column specifically chosen to prevent one. The error says to pass a string.
//!
//! ## Wire layout produced
//!
//! All words **big-endian**, matching the decoder:
//!
//! ```text
//! int16 ndigits, int16 weight, uint16 sign, int16 dscale, int16 digits[ndigits]
//! ```
//!
//! `NaN` is emitted as the sign word `0xC000` with no digits at all.

use crate::value::Value;

use super::super::error::DecodeError;
use super::mismatch;

#[path = "binary_encode_numeric_chunk.rs"]
mod chunk;
#[path = "binary_encode_numeric_groups.rs"]
mod groups;
#[path = "binary_encode_numeric_split.rs"]
mod split;

/// Sign word for a positive value.
const SIGN_POS: u16 = 0x0000;
/// Sign word for a negative value.
const SIGN_NEG: u16 = 0x4000;
/// Sign word for NaN, which carries no digit groups.
const SIGN_NAN: u16 = 0xC000;

/// Encode a `numeric` parameter from an exact decimal string.
///
/// # Arguments
///
/// * `value` — a [`Value::Str`] holding a decimal literal or `NaN`, or a
///   [`Value::Int`].
///
/// # Returns
///
/// The 8-byte header followed by `ndigits` big-endian digit groups.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a [`Value::Float`] (pass a string instead, to keep
/// the value exact), for an unparsable decimal, or for any other value kind.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{encode_param, numeric_to_string, oid};
/// use tetherscript::value::Value;
///
/// // A money value survives the round trip exactly, with its scale intact.
/// let bound = Value::Str(std::rc::Rc::new("19.99".into()));
/// let bytes = encode_param(oid::NUMERIC, &bound).unwrap().unwrap();
/// assert_eq!(numeric_to_string(&bytes).unwrap(), "19.99");
/// ```
pub(super) fn encode(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let text = match value {
        Value::Str(text) => text.as_ref().clone(),
        Value::Int(int) => int.to_string(),
        Value::Float(_) => {
            return Err(DecodeError::BadValue {
                what: "numeric",
                detail: "refusing to bind a float as numeric, which would lose exactness; \
                         pass the decimal as a string"
                    .into(),
            });
        }
        other => return Err(mismatch("numeric", other)),
    };
    if text.trim().eq_ignore_ascii_case("nan") {
        return Ok(header(0, 0, SIGN_NAN, 0));
    }
    let parsed = groups::parse(text.trim())?;
    let sign = if parsed.negative { SIGN_NEG } else { SIGN_POS };
    let mut out = header(
        parsed.groups.len() as i16,
        parsed.weight,
        sign,
        parsed.dscale,
    );
    for group in &parsed.groups {
        out.extend_from_slice(&(*group as i16).to_be_bytes());
    }
    Ok(out)
}

/// Build the 8-byte big-endian header.
fn header(ndigits: i16, weight: i16, sign: u16, dscale: i16) -> Vec<u8> {
    let mut out = Vec::with_capacity(8);
    out.extend_from_slice(&ndigits.to_be_bytes());
    out.extend_from_slice(&weight.to_be_bytes());
    out.extend_from_slice(&sign.to_be_bytes());
    out.extend_from_slice(&dscale.to_be_bytes());
    out
}
