//! # Constant-time octet-string comparison
//!
//! One responsibility: compare two octet strings without letting the *time* the
//! comparison takes depend on where they first differ.
//!
//! ## Why a short-circuiting compare leaks
//!
//! `a == b` on slices, and any hand-rolled loop with an early `return false`,
//! stops at the first mismatching octet. The time it takes therefore reveals the
//! length of the matching prefix. An attacker who can submit candidate
//! signatures and measure verification latency learns, one octet at a time,
//! whether the digest they induced agrees with the expected digest further than
//! their previous attempt. That turns an exponential search into a linear one.
//!
//! For a *public-key* verification of a *public* digest that leak is often
//! considered harmless. It is avoided anyway because:
//!
//! - the same routine gets reused for MAC tags and session tokens, where the
//!   compared value is a secret, and a leaky helper is the wrong thing to have
//!   lying around; and
//! - the expected digest is not always public — verifying a signature over a
//!   value the attacker is trying to guess (a bearer token, an account
//!   identifier) makes prefix timing directly useful.
//!
//! ## How this stays constant time
//!
//! Every octet pair is XORed and the results are accumulated with `|=`, so the
//! loop always runs to completion and the branch is taken once, on a value that
//! depends on the whole input. Lengths are compared first, which is fine: the
//! *lengths* are structural and already public here (a PKCS#1 v1.5 `DigestInfo`
//! has a fixed size per algorithm).
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::rsa::ct_eq;
//!
//! assert!(ct_eq(b"abcd", b"abcd"));
//! assert!(!ct_eq(b"abcd", b"abce"));
//! assert!(!ct_eq(b"abcd", b"abc"));
//! assert!(ct_eq(b"", b""));
//! ```

/// Compare two octet strings in time independent of their contents.
///
/// # Arguments
///
/// * `left` — first octet string.
/// * `right` — second octet string.
///
/// # Returns
///
/// `true` when the two slices have the same length and the same octets.
///
/// # Examples
///
/// ```rust
/// use tetherscript::rsa::ct_eq;
///
/// // A one-bit difference in the final octet is still detected.
/// assert!(!ct_eq(&[0x00, 0xff], &[0x00, 0xfe]));
/// ```
pub fn ct_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}
