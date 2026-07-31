//! The three password operations, independent of built-in registration.

use super::super::hmac::constant_time_eq;
use super::password_pbkdf2::{derive, HASH_LEN};
use super::password_phc::{encode, parse};
use super::password_salt::generate;

/// Default cost. OWASP's 2023 guidance for PBKDF2-HMAC-SHA-256 is 600,000.
pub(super) const DEFAULT_ITERATIONS: u32 = 600_000;

/// Hash `password` with a fresh salt at the default cost.
///
/// # Returns
///
/// A PHC-style string that records the algorithm, cost, and salt alongside the
/// digest, so no external metadata is needed to verify it later.
pub(super) fn hash(password: &str) -> String {
    let salt = generate();
    let digest = derive(password.as_bytes(), &salt, DEFAULT_ITERATIONS, HASH_LEN);
    encode(DEFAULT_ITERATIONS, &salt, &digest)
}

/// Verify `password` against a stored hash.
///
/// # Returns
///
/// True only when the recomputed digest matches.
///
/// # Errors
///
/// Propagates the parse error when `encoded` is malformed, so a corrupted record
/// is distinguishable from a wrong password. Returning `false` for both would hide
/// database corruption behind an apparent authentication failure.
pub(super) fn verify(password: &str, encoded: &str) -> Result<bool, String> {
    let stored = parse(encoded)?;
    let candidate = derive(
        password.as_bytes(),
        &stored.salt,
        stored.iterations,
        stored.hash.len().max(1),
    );
    // Constant-time: a byte-at-a-time compare would leak how much matched.
    Ok(constant_time_eq(&candidate, &stored.hash))
}

/// Report whether a stored hash was produced below `min_iterations`.
///
/// # Errors
///
/// Propagates the parse error when `encoded` is malformed.
pub(super) fn needs_rehash(encoded: &str, min_iterations: i64) -> Result<bool, String> {
    if min_iterations < 1 {
        return Err(format!(
            "password_needs_rehash: min_iterations must be at least 1, got {min_iterations}"
        ));
    }
    Ok(i64::from(parse(encoded)?.iterations) < min_iterations)
}
