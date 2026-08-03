//! # The 8-byte `numeric` header
//!
//! Four **big-endian** words: `ndigits`, `weight`, `sign`, `dscale`. Parsing is
//! separated from digit rendering so the validation rules live in one place.
//!
//! Every field is validated before it is trusted, because each one drives an
//! allocation or a loop bound:
//!
//! - `ndigits` is signed on the wire but can never be negative; a negative value cast
//!   to `usize` would become an enormous length.
//! - `dscale` likewise, and PostgreSQL caps it at **16383** (`NUMERIC_MAX_SCALE`), so a
//!   larger value is a corrupt frame rather than a very precise number.
//! - `sign` must be one of five documented words; anything else is rejected by name
//!   instead of being guessed as positive.
//!
//! The sign-word constants and the [`Header`] accessors are in the sibling
//! `binary_decode_numeric_sign.rs`.

use super::super::super::error::DecodeError;
use super::super::super::read::Reader;
use super::sign;

/// Highest `dscale` PostgreSQL will produce (`NUMERIC_MAX_SCALE`).
const MAX_DSCALE: i16 = 16_383;

/// A validated `numeric` header.
pub(super) struct Header {
    /// Number of base-10000 digit groups that follow. Always >= 0.
    pub(super) ndigits: usize,
    /// Base-10000 exponent of the first group; 0 means that group is the units.
    pub(super) weight: i16,
    /// One of the five documented sign words.
    pub(super) sign: u16,
    /// Decimal digits to display after the point. Always in 0..=16383.
    pub(super) dscale: usize,
}

/// Read and validate the header.
///
/// # Arguments
///
/// * `reader` — positioned at the start of the field body.
///
/// # Returns
///
/// The validated [`Header`], cursor advanced 8 bytes.
///
/// # Errors
///
/// [`DecodeError::Truncated`] on a short body, [`DecodeError::BadNumericSign`] for an
/// unknown sign word, and [`DecodeError::BadValue`] for a negative `ndigits`, a
/// negative `dscale`, or a `dscale` above the documented 16383 maximum.
pub(super) fn parse(reader: &mut Reader<'_>) -> Result<Header, DecodeError> {
    let ndigits = reader.i16("numeric ndigits")?;
    let weight = reader.i16("numeric weight")?;
    let sign_word = reader.u16("numeric sign")?;
    let dscale = reader.i16("numeric dscale")?;
    sign::validate(sign_word)?;
    if ndigits < 0 {
        return Err(bad(format!("negative digit-group count {ndigits}")));
    }
    if !(0..=MAX_DSCALE).contains(&dscale) {
        return Err(bad(format!(
            "display scale {dscale} outside 0..={MAX_DSCALE}"
        )));
    }
    Ok(Header {
        ndigits: ndigits as usize,
        weight,
        sign: sign_word,
        dscale: dscale as usize,
    })
}

/// Wrap a validation failure as a named `numeric` error.
fn bad(detail: String) -> DecodeError {
    DecodeError::BadValue {
        what: "numeric",
        detail,
    }
}
