//! Wording for the role-container variants of [`ClaimError`].
//!
//! One responsibility: render the `realm_access` / `resource_access` refusals.
//! Each message carries the *scope* — either `realm_access` or
//! `resource_access.<client>` — so an operator can tell which of a token's
//! several role arrays is malformed without decoding the token by hand.

use crate::jwtrs::error_claims::ClaimError;

/// Render a role-container rejection.
///
/// # Arguments
///
/// * `err` — The refusal to describe.
///
/// # Returns
///
/// `Some(message)` for the role variants, `None` for everything else, which
/// [`crate::jwtrs::error_text_claims::claim_text`] renders.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn roles_text(err: &ClaimError) -> Option<String> {
    Some(match err {
        ClaimError::RolesContainerNotObject { scope, found } => {
            format!("jwtrs: `{scope}` must be a JSON object, got {found}")
        }
        ClaimError::RolesNotArray { scope, found } => {
            format!("jwtrs: `{scope}.roles` must be an array, got {found}")
        }
        ClaimError::RoleNotString { scope, found } => {
            format!("jwtrs: `{scope}.roles` holds a non-string element of type {found}")
        }
        ClaimError::TooManyRoles {
            scope,
            count,
            limit,
        } => format!("jwtrs: `{scope}.roles` has {count} entries; limit is {limit}"),
        ClaimError::TooManyResourceClients { count, limit } => {
            format!("jwtrs: `resource_access` has {count} clients; limit is {limit}")
        }
        _ => return None,
    })
}
