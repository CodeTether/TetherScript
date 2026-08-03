//! Percent-encoding for OAuth query and form values.
//!
//! # Why a local copy
//!
//! `form_codec::encode` is `pub(crate)` and *would* be reachable, but it belongs to
//! another sub-agent's group, and this task must not couple to a file it cannot edit
//! if the two definitions ever need to diverge. They already differ in one respect
//! that matters here: this encoder serves both a query component and an
//! `application/x-www-form-urlencoded` body, and the two disagree about spaces.
//!
//! # Space handling
//!
//! * In a **query component** a space may be `%20` or `+`.
//! * In a **form body** the HTML spec says `+`, and some OAuth servers send that.
//!
//! `%20` is emitted in both cases: it is unambiguous, it is valid in a form body per
//! the URL Standard's `application/x-www-form-urlencoded` parser (which
//! percent-decodes after replacing `+`), and it avoids the classic bug where a
//! literal `+` in a value — common in email addresses, `user+tag@example.com` —
//! round-trips as a space.
//!
//! Everything outside the RFC 3986 unreserved set is escaped. That deliberately
//! includes `:` and `/`, because these values are *components* being placed into a
//! larger URL; leaving a `/` raw in a `redirect_uri` value would let the value alter
//! the structure of the URL it is embedded in.
//!
//! # Examples
//!
//! ```rust,ignore
//! assert_eq!(encode("openid profile"), "openid%20profile");
//! assert_eq!(encode("https://app/cb"), "https%3A%2F%2Fapp%2Fcb");
//! assert_eq!(encode("a-b_c.d~e"), "a-b_c.d~e");
//! ```

#[path = "oauth_percent_decode.rs"]
pub(crate) mod decode;

use super::pkce::unreserved;

/// Uppercase hex digit for a 4-bit nibble.
///
/// RFC 3986 §2.1 says producers should emit uppercase; decoders accept either.
fn digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'A' + nibble - 10) as char,
    }
}

/// Percent-encode everything outside the unreserved set.
///
/// # Arguments
///
/// * `input` — Raw value, as UTF-8 text.
///
/// # Returns
///
/// Encoded text safe to place in a URL query component or a form body.
pub(crate) fn encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        if unreserved(*byte) {
            out.push(*byte as char);
        } else {
            out.push('%');
            out.push(digit(byte >> 4));
            out.push(digit(byte & 0x0f));
        }
    }
    out
}
