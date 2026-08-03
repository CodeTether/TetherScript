//! Whether a key's declared `alg` permits a requested algorithm.
//!
//! One responsibility: the suitability predicate. Split out because both selection
//! strategies in [`crate::jwks::select`] need it and neither should own it.

use crate::jwks::alg::SigAlg;
use crate::jwks::key::RsaPublicKey;

/// Whether `key` may be used for `alg`.
///
/// # Arguments
///
/// * `key` — A key that already passed parse-time validation, so it is already
///   known to be permitted for signature verification at all.
/// * `alg` — The algorithm the token claims.
///
/// # Returns
///
/// `true` when the key declared no `alg` — unrestricted, per RFC 7517 §4.4 — or
/// declared exactly `alg`. A declared algorithm must match exactly, so an `RS512`
/// key can never verify an `RS256` token even though both are RSA-PKCS1: the
/// issuer stated the key's purpose and that statement is binding.
///
/// # Errors
///
/// Cannot fail.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn suits(key: &RsaPublicKey, alg: SigAlg) -> bool {
    key.alg.is_none_or(|declared| declared == alg)
}
