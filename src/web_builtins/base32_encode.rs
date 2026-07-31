//! RFC 4648 base32 encoding.
//!
//! Base32 packs 5 bits per character, so input is consumed in 5-byte groups that
//! become 8 characters. A partial trailing group is zero-padded to a character
//! boundary and then padded with `=` to a multiple of 8, which is what lets a
//! decoder recover the exact original length.

/// The RFC 4648 base32 alphabet: `A`-`Z` then `2`-`7`.
///
/// Digits `0`, `1`, and `8` are deliberately absent, which is why base32 survives
/// being read aloud or transcribed by hand.
pub(super) const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// Number of `=` characters that terminate a block for each partial input length.
///
/// Indexed by `input.len() % 5`; index 0 is unused because a whole group needs no
/// padding.
const PADDING: [usize; 5] = [0, 6, 4, 3, 1];

/// Encode bytes as uppercase base32 with `=` padding.
///
/// # Arguments
///
/// * `bytes` — Input to encode.
///
/// # Returns
///
/// A base32 string whose length is always a multiple of 8.
pub(super) fn encode(bytes: &[u8]) -> String {
    let mut out = encode_nopad(bytes);
    out.push_str(&"=".repeat(PADDING[bytes.len() % 5]));
    out
}

/// Encode bytes as uppercase base32 without trailing `=` padding.
///
/// # Arguments
///
/// * `bytes` — Input to encode.
///
/// # Returns
///
/// A base32 string carrying only the significant characters. TOTP secrets are
/// normally exchanged in this form.
pub(super) fn encode_nopad(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(5) * 8);
    for chunk in bytes.chunks(5) {
        // Left-align the group in a 40-bit window so a short chunk is
        // zero-padded on the right, per RFC 4648.
        let mut window: u64 = 0;
        for (index, byte) in chunk.iter().enumerate() {
            window |= (*byte as u64) << (32 - index * 8);
        }
        // Each input byte contributes 8 bits, and each output character 5.
        let characters = (chunk.len() * 8).div_ceil(5);
        for slot in 0..characters {
            let shift = 35 - slot * 5;
            out.push(ALPHABET[((window >> shift) & 0x1f) as usize] as char);
        }
    }
    out
}
