//! Shape validation of the `roles` claim.
//!
//! # Security: a role list must be a list of strings
//!
//! [`from_claims`] rejects a bare string in `roles`, rather than treating it as a
//! one-element list. The leniency is dangerous in both directions:
//!
//! * A string is iterable-shaped in enough languages that `"admin"` silently becomes
//!   `["a","d","m","i","n"]` in one layer and `["admin"]` in another. The two layers
//!   then disagree about what the caller holds, and the disagreement is invisible
//!   until one of them is the authorisation check.
//! * Accepting either shape means the *producer* is never forced to be consistent,
//!   so a token minted with `roles: "admin,editor"` looks accepted and grants the
//!   single nonexistent role `admin,editor` — the caller loses `editor` invisibly,
//!   or, with a `contains` check anywhere downstream, gains both.
//!
//! Refusing the shape makes the malformed token fail loudly at the boundary, which
//! is the only place it is cheap to fix.

use std::collections::HashMap;

use crate::value::Value;

/// Read the `roles` claim as a list of strings.
///
/// # Arguments
///
/// * `claims` — The verified claims map.
///
/// # Returns
///
/// The role names, or an empty vector when `roles` is absent or `nil`. Absent is a
/// caller with no roles, which is normal.
///
/// # Errors
///
/// Returns an error when `roles` is a str (see the module note on why that is not
/// silently wrapped), when it is any other non-list type, or when any element is not
/// a str.
pub(super) fn from_claims(claims: &HashMap<String, Value>) -> Result<Vec<String>, String> {
    let value = match claims.get("roles") {
        None | Some(Value::Nil) => return Ok(Vec::new()),
        Some(value) => value,
    };
    if let Value::Str(text) = value {
        return Err(format!(
            "identity_from_claims: `roles` must be a list of str, got the str `{text}`; \
             a single string is not accepted as a one-element list because layers \
             disagree about whether it means one role or its characters"
        ));
    }
    let Value::List(items) = value else {
        return Err(format!(
            "identity_from_claims: `roles` must be a list of str, got {}",
            value.type_name()
        ));
    };
    items.borrow().iter().map(element).collect()
}

/// Require one role-list element to be a str, naming what arrived instead.
fn element(item: &Value) -> Result<String, String> {
    match item {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!(
            "identity_from_claims: every `roles` entry must be str, got {}",
            other.type_name()
        )),
    }
}
