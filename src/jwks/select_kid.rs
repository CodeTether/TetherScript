//! Selection by exact `kid`.
//!
//! One responsibility: find the key carrying a requested `kid` and confirm the
//! match is suitable for the requested algorithm.
//!
//! # Security
//!
//! Two refusals, both deliberate:
//!
//! * **A miss is a hard error, never a fallback.** Falling back to "any key" on an
//!   unrecognised `kid` would let an attacker name a `kid` nobody published and
//!   still get a key tried, which turns key selection into key roulette.
//! * **A match is still checked against the algorithm.** A `kid` matching a key
//!   whose declared `alg` contradicts the token is refused, so `kid` cannot be used
//!   to steer a token onto a key the issuer scoped to a different algorithm.

use crate::jwks::alg::SigAlg;
use crate::jwks::error::JwksError;
use crate::jwks::key::RsaPublicKey;
use crate::jwks::select_suits::suits;

/// Select the key whose `kid` matches.
///
/// # Arguments
///
/// * `keys` — Validated keys, in document order.
/// * `wanted` — The attacker-controlled `kid` from the unverified token header.
/// * `alg` — The algorithm the token claims.
///
/// # Returns
///
/// The single matching, suitable key.
///
/// # Errors
///
/// Returns [`JwksError::UnknownKid`], listing the usable `kid`s, when nothing
/// matches — a mismatch after a key rotation is the most common cause and is
/// otherwise invisible. Returns [`JwksError::UnsuitableKey`] when the match's
/// declared `alg` contradicts the request.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn by_kid<'keys>(
    keys: &'keys [RsaPublicKey],
    wanted: &str,
    alg: SigAlg,
) -> Result<&'keys RsaPublicKey, JwksError> {
    let found = keys
        .iter()
        .find(|key| key.kid.as_deref() == Some(wanted))
        .ok_or_else(|| JwksError::UnknownKid {
            kid: wanted.to_string(),
            available: keys.iter().filter_map(|key| key.kid.clone()).collect(),
        })?;
    if suits(found, alg) {
        return Ok(found);
    }
    Err(JwksError::UnsuitableKey {
        kid: wanted.to_string(),
        reason: format!(
            "its `alg` is `{}`, but the token claims `{}`",
            found.alg.map_or("<unset>", SigAlg::name),
            alg.name()
        ),
    })
}
