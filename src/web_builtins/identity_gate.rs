//! The role gate: continue, or refuse with 403.
//!
//! Separated from [`super::identity_response`] so the decision and the response
//! shape are independent: the status semantics are documented there, the
//! precondition is enforced here.

use super::identity_response;
use super::identity_roles;
use crate::value::Value;

/// Require an identity to hold a role.
///
/// # Arguments
///
/// * `identity` — An identity map from `identity_from_claims` or `anonymous`.
/// * `role` — The role the route requires.
///
/// # Returns
///
/// `Value::Nil` when the role is held, meaning the handler continues. Otherwise the
/// 403 response map from [`identity_response::forbidden`], which the handler
/// returns as-is.
///
/// Returning `nil` for success rather than `true` is deliberate: the call site reads
/// `if denied != nil { return denied }`, so the value that must be returned *is* the
/// value that was tested. A boolean would leave the handler to construct the refusal
/// itself, which is how one route ends up answering a different status than the
/// rest.
///
/// # Errors
///
/// Returns an error when `identity` is not a map, or when it is not authenticated.
/// An unauthenticated caller is not a 403 case — see [`identity_response`] for why
/// answering 403 there hides that a credential would have helped — so it surfaces as
/// a named error and the handler chooses between a 401 challenge and anonymous
/// access.
pub(super) fn require(identity: &Value, role: &str) -> Result<Value, String> {
    let Value::Map(map) = identity else {
        return Err(format!(
            "require_role: identity must be a map, got {}",
            identity.type_name()
        ));
    };
    let authenticated = matches!(map.borrow().get("authenticated"), Some(Value::Bool(true)));
    if !authenticated {
        return Err(format!(
            "require_role: caller is not authenticated, so role `{role}` cannot be \
             evaluated; answer 401 with a challenge or treat the route as anonymous"
        ));
    }
    if identity_roles::holds(identity, role) {
        return Ok(Value::Nil);
    }
    Ok(identity_response::forbidden(role))
}
