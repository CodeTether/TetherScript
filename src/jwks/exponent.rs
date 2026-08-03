//! Public-exponent acceptance rules for an RSA key from a JWKS document.
//!
//! One responsibility: decide whether a decoded `e` is a usable public exponent.
//!
//! # Security: why each rule is here
//!
//! * **`e = 0` rejected.** `m^0 mod n` is 1 for every `m`, so the "signature" of
//!   every message is the same constant — a key that verifies anything.
//! * **`e = 1` rejected.** `m^1 mod n` is `m`, so the signature *is* the padded
//!   digest. Anyone who can compute the digest can produce the signature, which is
//!   every attacker.
//! * **Even exponent rejected.** `e` must be coprime to `(p-1)(q-1)`, which is
//!   even, so a valid `e` is odd. An even one cannot be a real RSA exponent.
//! * **Leading zero byte rejected.** Same minimal-encoding rule as the modulus:
//!   one key must have one spelling.
//! * **Size ceiling.** Verification cost grows with the exponent's bit length, and
//!   no real key uses more than a few bytes (65537 is 3).

use crate::jwks::bits::bit_length;
use crate::jwks::limits::MAX_EXPONENT_BYTES;

/// Validate a decoded RSA public exponent.
///
/// # Arguments
///
/// * `exponent` — Decoded big-endian `e` bytes.
/// * `label` — Locating name used in error text.
///
/// # Returns
///
/// `Ok(())` when the exponent is usable.
///
/// # Errors
///
/// Returns a named error when `exponent` is empty, begins with a zero byte, is
/// longer than [`MAX_EXPONENT_BYTES`], is even, or is 0 or 1.
///
/// # Panics
///
/// Does not panic.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::exponent::check;
///
/// assert!(check(&[0x01, 0x00, 0x01], "k").is_ok()); // 65537
/// assert!(check(&[0x03], "k").is_ok());
/// assert!(check(&[0x01], "k").unwrap_err().contains("must be at least 3"));
/// assert!(check(&[0x00], "k").is_err());
/// assert!(check(&[], "k").is_err());
/// assert!(check(&[0x04], "k").unwrap_err().contains("even"));
/// ```
pub fn check(exponent: &[u8], label: &str) -> Result<(), String> {
    let Some(first) = exponent.first() else {
        return Err(format!("{label}: exponent `e` is empty"));
    };
    if *first == 0 {
        return Err(format!(
            "{label}: exponent `e` has a leading zero byte; RFC 7518 requires the \
             minimal big-endian encoding"
        ));
    }
    if exponent.len() > MAX_EXPONENT_BYTES {
        return Err(format!(
            "{label}: exponent `e` is {} bytes; limit is {MAX_EXPONENT_BYTES}",
            exponent.len()
        ));
    }
    if exponent.last().is_some_and(|last| last % 2 == 0) {
        return Err(format!(
            "{label}: exponent `e` is even, so it cannot be coprime to (p-1)(q-1)"
        ));
    }
    if bit_length(exponent) < 2 {
        return Err(format!(
            "{label}: exponent `e` is 1, which makes a signature equal to the \
             padded digest; `e` must be at least 3"
        ));
    }
    Ok(())
}
