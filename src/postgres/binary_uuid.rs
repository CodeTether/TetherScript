//! # Canonical UUID rendering and parsing
//!
//! A binary `uuid` is 16 raw bytes in network order with no internal structure the
//! wire cares about. Scripts want the canonical hyphenated form, so the 16 bytes are
//! rendered as lowercase hex split `8-4-4-4-12`, and the reverse direction exists so
//! a `uuid` parameter can be bound in binary format.
//!
//! Both directions live in one file because they are two halves of one
//! responsibility: they must agree, and a round-trip test proves it.
//!
//! Hex is written by hand rather than pulled from a crate: this is the
//! zero-dependency core, and 16 bytes of hex is a handful of lines.

/// Render 16 bytes as a canonical lowercase hyphenated UUID.
///
/// # Arguments
///
/// * `bytes` — the 16-byte body. A shorter slice simply produces a shorter string,
///   so this cannot panic; callers validate the length before calling.
///
/// # Returns
///
/// `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` in lowercase.
pub(super) fn hyphenate(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(36);
    for (index, byte) in bytes.iter().enumerate() {
        // Hyphens fall after bytes 4, 6, 8, and 10 — the 8-4-4-4-12 split.
        if matches!(index, 4 | 6 | 8 | 10) {
            out.push('-');
        }
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Parse a canonical or unhyphenated UUID string into 16 bytes.
///
/// # Arguments
///
/// * `text` — a UUID with or without hyphens, in either letter case.
///
/// # Returns
///
/// `Some([u8; 16])` when the text holds exactly 32 hex digits, `None` otherwise.
/// `None` is a *caller* error — a bad parameter value — not a wire error, so it is an
/// `Option` here and the encoder turns it into a named error with the offending text.
pub(super) fn parse(text: &str) -> Option<[u8; 16]> {
    let digits: Vec<u8> = text.bytes().filter(|byte| *byte != b'-').collect();
    if digits.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for (index, pair) in digits.chunks(2).enumerate() {
        let high = (pair[0] as char).to_digit(16)?;
        let low = (pair[1] as char).to_digit(16)?;
        out[index] = (high * 16 + low) as u8;
    }
    Some(out)
}
