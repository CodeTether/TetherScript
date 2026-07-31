//! Structural validation for base32 input.
//!
//! Split from `base32_decode.rs` to respect the 50-line limit. These checks are
//! what make decoding strict: a malformed secret is reported rather than silently
//! truncated into a key that only fails much later, at authentication time.

/// Remove trailing `=` padding, rejecting any that appears mid-string.
///
/// Padding is optional, but when present it must bring the block to a multiple of
/// eight characters. A run of the wrong length means a truncated or concatenated
/// value, not merely cosmetic sloppiness.
///
/// # Arguments
///
/// * `upper` — Case-folded base32 text.
///
/// # Returns
///
/// The significant characters, with padding removed.
///
/// # Errors
///
/// Returns an error naming the position when `=` appears before the end, or when
/// a padding run does not pad the block to a multiple of eight characters.
pub(super) fn strip_padding(upper: &str) -> Result<&str, String> {
    let body = upper.trim_end_matches('=');
    if let Some(position) = body.find('=') {
        return Err(format!(
            "base32_decode: unexpected padding `=` at position {position}"
        ));
    }
    let padding = upper.len() - body.len();
    // Unpadded input is accepted, so only a present run is length-checked.
    if padding > 0 && !upper.len().is_multiple_of(8) {
        return Err(format!(
            "base32_decode: {padding} padding characters leave {} total, not a multiple of 8",
            upper.len()
        ));
    }
    Ok(body)
}

/// Map one alphabet character to its 5-bit value.
///
/// # Arguments
///
/// * `character` — Candidate character, already upper-cased.
/// * `position` — Zero-based index, used only for the error message.
///
/// # Returns
///
/// The 5-bit value in `0..32`.
///
/// # Errors
///
/// Returns an error naming the character and its position when it is outside the
/// RFC 4648 alphabet.
pub(super) fn quintet(character: char, position: usize) -> Result<u8, String> {
    match character {
        'A'..='Z' => Ok(character as u8 - b'A'),
        '2'..='7' => Ok(character as u8 - b'2' + 26),
        other => Err(format!(
            "base32_decode: invalid character `{other}` at position {position}"
        )),
    }
}

/// Report whether a significant-character count can form whole bytes.
///
/// # Arguments
///
/// * `count` — Number of significant characters.
///
/// # Returns
///
/// True when `count % 8` is a remainder RFC 4648 can produce. Remainders 1, 3,
/// and 6 are impossible and indicate a truncated value.
pub(super) fn is_valid_length(count: usize) -> bool {
    matches!(count % 8, 0 | 2 | 4 | 5 | 7)
}

/// Reject non-zero discarded bits, which mean the input was not canonical.
///
/// # Arguments
///
/// * `quintets` — Decoded 5-bit groups.
/// * `count` — Number of significant characters.
///
/// # Errors
///
/// Returns an error naming the final position when its unused low bits are set.
/// Accepting those would let two different texts decode to the same bytes.
pub(super) fn check_tail_bits(quintets: &[u8], count: usize) -> Result<(), String> {
    let unused = (count * 5) % 8;
    if unused == 0 {
        return Ok(());
    }
    let last = *quintets.last().unwrap_or(&0);
    if last & ((1 << unused) - 1) != 0 {
        return Err(format!(
            "base32_decode: final character at position {} has {unused} non-zero unused bits",
            count - 1
        ));
    }
    Ok(())
}
