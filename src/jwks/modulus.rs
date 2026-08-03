//! Modulus acceptance rules for an RSA public key from a JWKS document.
//!
//! One responsibility: decide whether a decoded `n` is a modulus worth handing to
//! a verifier, and report its size.
//!
//! # Security: why each rule is here
//!
//! * **Leading zero byte rejected.** RFC 7518 §6.3.1.1 requires the octet string
//!   to be the minimal big-endian encoding. Accepting `00 ab cd…` gives one key
//!   two spellings, so a cache or a "have I seen this key" comparison keyed on the
//!   bytes can be split, and a size check that counts bytes rather than bits can
//!   be tricked into thinking a 2040-bit modulus is 2048.
//! * **Even modulus rejected.** A real RSA modulus is a product of two odd primes
//!   and is therefore odd. An even modulus is either corruption or a crafted value
//!   with known factorisation — i.e. a key an attacker can sign for.
//! * **Size floor.** 1024-bit RSA is not safe for new signatures, and a JWKS is
//!   exactly where a downgrade would be smuggled in.
//! * **Size ceiling.** Modular exponentiation cost grows quadratically in modulus
//!   size, so an absurd modulus is a CPU-exhaustion vector on an unauthenticated
//!   path.

use crate::jwks::bits::bit_length;
use crate::jwks::limits::{MAX_MODULUS_BYTES, MIN_MODULUS_BITS, MIN_MODULUS_BYTES};

/// Validate a decoded RSA modulus.
///
/// # Arguments
///
/// * `modulus` — Decoded big-endian `n` bytes.
/// * `label` — Locating name used in error text.
///
/// # Returns
///
/// The modulus size in significant bits.
///
/// # Errors
///
/// Returns a named error when `modulus` is empty, begins with a zero byte, is
/// even, is shorter than [`MIN_MODULUS_BYTES`] or [`MIN_MODULUS_BITS`], or is
/// longer than [`MAX_MODULUS_BYTES`].
///
/// # Panics
///
/// Does not panic.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::modulus::check;
///
/// let mut good = vec![0xff; 256];
/// good[255] = 0xf3; // odd
/// assert_eq!(check(&good, "k").unwrap(), 2048);
///
/// let mut leading_zero = good.clone();
/// leading_zero.insert(0, 0x00);
/// assert!(check(&leading_zero, "k").unwrap_err().contains("leading zero"));
///
/// let mut even = good.clone();
/// even[255] = 0xf2;
/// assert!(check(&even, "k").unwrap_err().contains("even"));
/// ```
pub fn check(modulus: &[u8], label: &str) -> Result<usize, String> {
    let Some(first) = modulus.first() else {
        return Err(format!("{label}: modulus `n` is empty"));
    };
    if *first == 0 {
        return Err(format!(
            "{label}: modulus `n` has a leading zero byte; RFC 7518 requires the \
             minimal big-endian encoding"
        ));
    }
    if modulus.last().is_some_and(|last| last % 2 == 0) {
        return Err(format!(
            "{label}: modulus `n` is even, so it cannot be a product of two odd primes"
        ));
    }
    if modulus.len() > MAX_MODULUS_BYTES {
        return Err(format!(
            "{label}: modulus `n` is {} bytes; limit is {MAX_MODULUS_BYTES}",
            modulus.len()
        ));
    }
    let bits = bit_length(modulus);
    if modulus.len() < MIN_MODULUS_BYTES || bits < MIN_MODULUS_BITS {
        return Err(format!(
            "{label}: modulus `n` is {bits} bits; at least {MIN_MODULUS_BITS} required"
        ));
    }
    Ok(bits)
}
