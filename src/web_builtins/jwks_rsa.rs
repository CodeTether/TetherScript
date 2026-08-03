//! RSA key-material validation for one JWK.
//!
//! One responsibility: decide whether a decoded JWK is an RSA public key this
//! deployment is willing to use, and report the modulus size.
//!
//! # Security
//!
//! Three refusals live here, and all three are refusals *at parse time* rather
//! than at use time, so a weak key can never reach a verifier:
//!
//! 1. **`kty` must be `RSA`.** An `oct` (symmetric) entry in a JWKS document is
//!    either a misconfiguration or an attempt to get an HMAC secret treated as a
//!    public key, which is the RS256-to-HS256 confusion attack in key form.
//! 2. **The modulus must be at least 2048 bits.** 1024-bit RSA is considered
//!    broken for new signatures; accepting it silently would make the whole
//!    verification chain only as strong as its weakest published key.
//! 3. **The exponent must be non-empty.** An empty `e` decodes to zero, and
//!    modular exponentiation by zero yields 1 for every signature — i.e. a key
//!    that "verifies" anything.

/// Minimum accepted modulus length in bytes (2048 bits).
pub(super) const MIN_MODULUS_BYTES: usize = 256;

/// Require that `kty` names an RSA key.
///
/// # Arguments
///
/// * `kty` — The JWK `kty` member.
/// * `label` — Qualified name used in error text.
///
/// # Returns
///
/// `Ok(())` when `kty` is exactly `RSA`.
///
/// # Errors
///
/// Returns a named error naming the rejected `kty` otherwise. The comparison is
/// case-sensitive because RFC 7518 registers the value as `RSA`.
///
/// # Examples
///
/// ```tether
/// let doc = "{\"keys\":[{\"kty\":\"oct\",\"kid\":\"a\",\"k\":\"AAAA\"}]}"
/// println(str(jwks_parse(doc).is_err()))   // true
/// ```
pub(super) fn require_rsa(kty: &str, label: &str) -> Result<(), String> {
    if kty == "RSA" {
        return Ok(());
    }
    Err(format!(
        "{label}: unsupported kty `{kty}`; only RSA public keys are accepted"
    ))
}

/// Reject a modulus or exponent that cannot be used safely.
///
/// # Arguments
///
/// * `modulus` — Decoded big-endian `n` bytes.
/// * `exponent` — Decoded big-endian `e` bytes.
/// * `label` — Qualified name used in error text.
///
/// # Returns
///
/// The modulus size in bits, counted from its most significant set bit so a
/// leading zero byte cannot inflate the reported strength.
///
/// # Errors
///
/// Returns a named error when the modulus is under [`MIN_MODULUS_BYTES`] or the
/// exponent is empty.
///
/// # Examples
///
/// ```tether
/// // A 1024-bit modulus is refused even though it is well-formed base64url.
/// println(str(jwks_parse(weak_doc).is_err()))   // true
/// ```
pub(super) fn check_material(modulus: &[u8], exponent: &[u8], label: &str) -> Result<i64, String> {
    if modulus.len() < MIN_MODULUS_BYTES {
        return Err(format!(
            "{label}: modulus is {} bytes; at least {MIN_MODULUS_BYTES} (2048-bit) required",
            modulus.len()
        ));
    }
    if exponent.is_empty() {
        return Err(format!("{label}: exponent `e` is empty"));
    }
    Ok(bit_length(modulus))
}

/// Count significant bits in a big-endian integer.
fn bit_length(bytes: &[u8]) -> i64 {
    let leading_zeros = bytes.iter().take_while(|byte| **byte == 0).count();
    match bytes.get(leading_zeros) {
        None => 0,
        Some(top) => {
            let tail = (bytes.len() - leading_zeros - 1) as i64 * 8;
            tail + (8 - top.leading_zeros() as i64)
        }
    }
}
