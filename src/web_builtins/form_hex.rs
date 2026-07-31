//! Hex nibble conversion for percent escapes.
//!
//! Its own file so [`super::form_codec`] stays within the 50-line limit and the
//! digit tables are testable in isolation.

/// Render a 4-bit nibble as an uppercase hex digit.
///
/// RFC 3986 says producers should emit uppercase; decoders accept either case.
pub(crate) fn digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'A' + nibble - 10) as char,
    }
}

/// Parse one hex digit, accepting upper or lower case.
///
/// # Returns
///
/// The 0-15 value, or `None` when `byte` is not a hex digit.
pub(crate) fn value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
