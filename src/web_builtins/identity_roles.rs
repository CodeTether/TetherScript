//! Role membership: exact comparison.
//!
//! # Security: exact comparison, never prefix or substring
//!
//! [`holds`] compares whole strings with `==`. That is the entire mechanism, and it
//! is worth naming what the tempting alternatives break:
//!
//! * `starts_with` — `has_role(id, "admin")` would succeed for a caller holding
//!   `administrator-readonly`. A deliberately *narrowed* role becomes a superset of
//!   the privileged one, so the least-privilege grant escalates.
//! * `contains` — `has_role(id, "admin")` would succeed for a caller holding
//!   `not-admin`, `admin-denied`, or `pending-admin-approval`. Every naming
//!   convention that encodes a negation or a state becomes an authorisation bypass.
//! * `eq_ignore_ascii_case` — two roles differing only in case become one, so a
//!   directory that treats `Admin` and `admin` as distinct principals disagrees with
//!   the gate. Roles are opaque identifiers, not prose.
//!
//! The role-list *shape* is validated in [`super::identity_roles_claim`].

use crate::value::Value;

/// Whether `role` is among an identity's roles, by exact match.
///
/// # Arguments
///
/// * `identity` — An identity map, as produced by `identity_from_claims`.
/// * `role` — The role name to test.
///
/// # Returns
///
/// `true` only when the identity is authenticated **and** its `roles` list contains
/// an entry equal to `role`. An unauthenticated identity always answers `false`, so
/// a caller cannot hold a role without being someone. A malformed or absent `roles`
/// field answers `false` rather than erroring: a bool built-in has no error channel,
/// and `false` is the fail-closed answer.
pub(super) fn holds(identity: &Value, role: &str) -> bool {
    let Value::Map(map) = identity else {
        return false;
    };
    let map = map.borrow();
    if !matches!(map.get("authenticated"), Some(Value::Bool(true))) {
        return false;
    }
    let Some(Value::List(roles)) = map.get("roles") else {
        return false;
    };
    roles
        .borrow()
        .iter()
        .any(|held| matches!(held, Value::Str(text) if text.as_str() == role))
}
