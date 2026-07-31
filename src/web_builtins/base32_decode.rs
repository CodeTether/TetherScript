//! RFC 4648 base32 decoding.
//!
//! Accepts either case and tolerates missing padding, so a secret copied from an
//! authenticator app round-trips whether or not the `=` characters came along.
//! Structural checks live in [`super::base32_validate`].

use super::base32_validate::{check_tail_bits, is_valid_length, quintet, strip_padding};

/// Decode base32 text, accepting either case.
///
/// # Arguments
///
/// * `input` — Base32 text. Lower case is folded to upper case, and trailing `=`
///   padding is optional.
///
/// # Returns
///
/// The decoded bytes.
///
/// # Errors
///
/// Returns an error naming the character and zero-based position when `input`
/// holds a non-alphabet character, when `=` appears before the end, when the
/// significant length is impossible for base32, or when the unused tail bits of
/// the final character are non-zero.
pub(super) fn decode(input: &str) -> Result<Vec<u8>, String> {
    let upper = input.to_ascii_uppercase();
    let body = strip_padding(&upper)?;
    let mut quintets = Vec::with_capacity(body.len());
    for (position, character) in body.chars().enumerate() {
        quintets.push(quintet(character, position)?);
    }
    if !is_valid_length(quintets.len()) {
        return Err(format!(
            "base32_decode: {} significant characters cannot form whole bytes",
            quintets.len()
        ));
    }
    check_tail_bits(&quintets, body.len())?;
    Ok(pack(&quintets))
}

/// Concatenate 5-bit groups and emit whole bytes, discarding the partial tail.
fn pack(quintets: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(quintets.len() * 5 / 8);
    let mut buffer: u16 = 0;
    let mut bits = 0u32;
    for quintet in quintets {
        buffer = (buffer << 5) | *quintet as u16;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
        }
    }
    out
}
