//! Selection when the token carries no `kid`.
//!
//! One responsibility: select the *unique* suitable key, or refuse.
//!
//! # Security: a tie is refused, not broken
//!
//! `kid` is optional in a JWS header, so a token can legitimately arrive without
//! one. When it does, this module selects only if **exactly one** published key is
//! suitable. Several suitable keys yields [`JwksError::AmbiguousKey`].
//!
//! The alternatives are all worse:
//!
//! * *Pick the first.* Then the effective signing key depends on the issuer's
//!   document order, which the issuer does not consider load-bearing and may
//!   change on any rotation.
//! * *Pick the newest.* A JWK has no ordering or issuance field to read, so this
//!   is document order under another name.
//! * *Try each in turn.* This turns one token into N verifications — an
//!   amplification vector on an unauthenticated path — and means a token verified
//!   by *any* key is accepted, so retiring a compromised key stops working.
//!
//! Refusing makes the operator's choice explicit: either the token should carry a
//! `kid`, or the realm should publish one signing key.

use crate::jwks::alg::SigAlg;
use crate::jwks::error::JwksError;
use crate::jwks::key::RsaPublicKey;
use crate::jwks::select_suits::suits;

/// Select the one suitable key.
///
/// # Arguments
///
/// * `keys` — Validated keys, in document order.
/// * `alg` — The algorithm the token claims.
///
/// # Returns
///
/// The single suitable key.
///
/// # Errors
///
/// Returns [`JwksError::NoSuitableKey`] when nothing is suitable, and
/// [`JwksError::AmbiguousKey`], naming every candidate, when more than one is.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn unique(keys: &[RsaPublicKey], alg: SigAlg) -> Result<&RsaPublicKey, JwksError> {
    let mut candidates = keys.iter().filter(|key| suits(key, alg));
    let first = candidates.next().ok_or_else(|| JwksError::NoSuitableKey {
        alg: alg.name().to_string(),
    })?;
    if candidates.next().is_none() {
        return Ok(first);
    }
    Err(JwksError::AmbiguousKey {
        alg: alg.name().to_string(),
        candidates: keys
            .iter()
            .filter(|key| suits(key, alg))
            .map(|key| key.kid.clone().unwrap_or_else(|| "<no kid>".into()))
            .collect(),
    })
}
