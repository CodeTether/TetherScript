//! Key selection: dispatch to `kid` lookup or unique-suitability.
//!
//! One responsibility: choose which selection strategy applies. The strategies
//! themselves live in `crate::jwks::select_kid` and
//! `crate::jwks::select_unique`, and the suitability predicate they share lives
//! in `crate::jwks::select_suits`.
//!
//! # Security: `kid` is attacker-controlled
//!
//! The `kid` passed here is read from a JWT header that has **not** been verified
//! yet — finding the key that will verify it is the whole point of selection. So
//! `kid` is an arbitrary attacker-chosen string of arbitrary content, bounded only
//! by [`MAX_FIELD_CHARS`](crate::jwks::limits::MAX_FIELD_CHARS).
//!
//! Using it to *select* is safe, because a lookup can only ever return a key the
//! issuer published: an attacker can steer which trusted key is tried, but cannot
//! introduce an untrusted one. What is **not** safe:
//!
//! * **Never build a filesystem path from `kid`.** `../../etc/passwd` is a valid
//!   `kid`, so `keys/{kid}.pem` is a path-traversal bug.
//! * **Never use `kid` as a bare cache key.** Its cardinality is unbounded, so
//!   varying it evicts or grows a cache without limit. Hash it and bound the cache.
//! * **Never sanitise by trusting it.** If `kid` must appear in a log line, a
//!   metric label, or a filename, escape or hash it at that boundary.

use crate::jwks::alg::SigAlg;
use crate::jwks::error::JwksError;
use crate::jwks::key::RsaPublicKey;
use crate::jwks::select_kid::by_kid;
use crate::jwks::select_unique::unique;

/// Select the key to verify with.
///
/// # Arguments
///
/// * `keys` — Validated keys, in document order.
/// * `kid` — The `kid` from the unverified token header, if it carried one.
/// * `alg` — The algorithm the token claims, already parsed to a closed set.
///
/// # Returns
///
/// A reference to the single selected key.
///
/// # Errors
///
/// Returns [`JwksError::UnknownKid`] or [`JwksError::UnsuitableKey`] when a `kid`
/// was requested, and [`JwksError::NoSuitableKey`] or [`JwksError::AmbiguousKey`]
/// when one was not.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn select<'keys>(
    keys: &'keys [RsaPublicKey],
    kid: Option<&str>,
    alg: SigAlg,
) -> Result<&'keys RsaPublicKey, JwksError> {
    match kid {
        Some(wanted) => by_kid(keys, wanted, alg),
        None => unique(keys, alg),
    }
}
