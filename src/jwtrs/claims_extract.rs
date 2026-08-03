//! Filling a [`Claims`] from an authenticated payload.
//!
//! One responsibility: the field-by-field extraction. Split from
//! [`crate::jwtrs::claims`] so that file is the type and its guarantee, and this
//! one is the mechanical read.
//!
//! # Why `iss` and `sub` are required
//!
//! `iss` is required because the *comparison* against the configured issuer is only
//! meaningful if the claim exists; an absent `iss` cannot match anything and is
//! rejected as missing rather than as a mismatch, which is a clearer diagnosis.
//! `sub` is required because a token that identifies no subject cannot support an
//! authorization decision or an audit trail.

use crate::jwtrs::audience;
use crate::jwtrs::authenticated::Authenticated;
use crate::jwtrs::claims::Claims;
use crate::jwtrs::error_claims::ClaimError;
use crate::jwtrs::payload_fields::{optional_secs, optional_str, required_secs, required_str};
use crate::jwtrs::{realm_roles, resource_roles};

/// Read every claim this module models.
///
/// # Arguments
///
/// * `token` — A payload whose signature a verifier accepted.
///
/// # Returns
///
/// The populated claim set. Note that no *policy* is applied here: issuer and
/// audience matching and the time window are [`crate::jwtrs::validate`]'s job.
///
/// # Errors
///
/// [`ClaimError::Missing`] for an absent `iss`, `sub`, or `exp`, the `NotString` /
/// `NotNumber` variants for wrong-typed claims, and every role and audience error.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn extract(token: &Authenticated) -> Result<Claims, ClaimError> {
    let members = token.payload();
    Ok(Claims {
        iss: required_str(members, "iss")?,
        sub: required_str(members, "sub")?,
        aud: audience::read(members)?,
        exp: required_secs(members, "exp")?,
        nbf: optional_secs(members, "nbf")?,
        iat: optional_secs(members, "iat")?,
        azp: optional_str(members, "azp")?,
        realm_roles: realm_roles::read(members)?,
        resource_roles: resource_roles::read(members)?,
        kid: token.header.kid.clone(),
    })
}
