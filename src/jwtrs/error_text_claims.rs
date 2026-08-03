//! Wording for the non-role variants of [`ClaimError`].
//!
//! One responsibility: render issuer, audience, type, and time-window refusals.
//! Role-container wording lives in [`crate::jwtrs::error_text_roles`] so neither
//! file grows past the limit.

use crate::jwtrs::error_claims::ClaimError;

/// Render a claim-stage rejection that is not about roles.
///
/// # Arguments
///
/// * `err` — The refusal to describe.
///
/// # Returns
///
/// `Some(message)` for the variants this file owns, `None` for the role
/// variants, which [`crate::jwtrs::error_text_roles::roles_text`] renders.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn claim_text(err: &ClaimError) -> Option<String> {
    Some(match err {
        ClaimError::Missing(name) => format!(
            "jwtrs: required claim `{name}` is absent; \
             a token with no `{name}` is rejected rather than treated as unbounded"
        ),
        ClaimError::NotString { name, found } => {
            format!("jwtrs: claim `{name}` must be a string, got {found}")
        }
        ClaimError::NotNumber { name, found } => {
            format!("jwtrs: claim `{name}` must be a number, got {found}")
        }
        ClaimError::IssuerMismatch { got, expected } => {
            format!("jwtrs: claim `iss` is `{got}`, expected `{expected}`")
        }
        ClaimError::AudienceNotStringOrArray(found) => {
            format!("jwtrs: claim `aud` must be a string or an array of strings, got {found}")
        }
        ClaimError::TooManyAudiences { count, limit } => {
            format!("jwtrs: claim `aud` has {count} entries; limit is {limit}")
        }
        ClaimError::AudienceMismatch { got, expected } => format!(
            "jwtrs: claim `aud` is [{}] and matches none of the accepted audiences [{}]; \
             this token was minted for another service",
            got.join(", "),
            expected.join(", ")
        ),
        ClaimError::Expired { exp, now, skew } => {
            format!("jwtrs: token expired at {exp}; now is {now} with {skew}s skew allowed")
        }
        ClaimError::NotYetValid { nbf, now, skew } => format!(
            "jwtrs: token is not valid before {nbf}; now is {now} with {skew}s skew allowed"
        ),
        _ => return None,
    })
}
