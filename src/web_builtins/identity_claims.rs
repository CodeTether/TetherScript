//! Turning a verified claim set into an identity.
//!
//! # Trust boundary
//!
//! This module starts *after* verification. It never inspects a signature, and it
//! must only be handed the output of something that did — `jwt_verify`,
//! `session_open`, or an equivalent. Handing it a decoded-but-unverified payload
//! makes every field below attacker-chosen, which is precisely the bug the split
//! exists to keep visible: extraction and verification are different jobs, and
//! credential extraction stays in `bearer_token`.
//!
//! # Anonymous by default
//!
//! Absent claims (`nil`) and an empty claim map both yield
//! [`super::identity_shape::anonymous`] with `authenticated: false`. Neither is an
//! error, because "no credential presented" is an ordinary request, and forcing a
//! handler to branch on an error to discover it invites the handler to `?` past it.
//! What is *not* possible is an authenticated identity from nothing: see
//! [`super::identity_shape`] for why that is structural rather than a convention.
//!
//! An `authenticated` field inside the claims is deliberately ignored.

use std::collections::HashMap;

use super::identity_headers::as_map;
use super::identity_roles_claim;
use super::identity_shape;
use crate::value::Value;

/// Convert verified claims into an identity map.
///
/// # Arguments
///
/// * `claims` — A claims map, or `nil` when no credential was presented.
///
/// # Returns
///
/// An authenticated identity when the claims name a subject in `sub` (or
/// `subject`), otherwise the anonymous identity.
///
/// # Errors
///
/// Returns an error naming the offending field when `claims` is neither a map nor
/// `nil`, when the subject field is present but not a str, or when `roles` is
/// present but is not a list of strings — including the case where it is a single
/// string. See [`identity_roles_claim`] for why that leniency is refused.
pub(super) fn from_claims(claims: &Value) -> Result<Value, String> {
    // `nil` is "no credential", not a type error.
    if matches!(claims, Value::Nil) {
        return Ok(identity_shape::anonymous());
    }
    let map = as_map(claims, "identity_from_claims: claims")?;
    if map.is_empty() {
        return Ok(identity_shape::anonymous());
    }

    let subject = subject_of(&map)?;
    let roles = identity_roles_claim::from_claims(&map)?;
    // A claim set with roles but no subject stays anonymous rather than becoming a
    // role-bearing nobody; `identity` drops the roles in that case.
    Ok(identity_shape::identity(subject.as_deref(), roles))
}

/// Read the subject, preferring the JWT-standard `sub`.
///
/// # Errors
///
/// Returns an error naming the field when it is present but not a str. A non-str
/// subject is a malformed token, and coercing an int user id to a string here would
/// make `1` and `"1"` the same principal in one code path and not another.
fn subject_of(claims: &HashMap<String, Value>) -> Result<Option<String>, String> {
    for field in ["sub", "subject"] {
        match claims.get(field) {
            Some(Value::Str(text)) => return Ok(Some((**text).clone())),
            Some(other) => {
                return Err(format!(
                    "identity_from_claims: `{field}` must be str, got {}",
                    other.type_name()
                ));
            }
            None => continue,
        }
    }
    Ok(None)
}
