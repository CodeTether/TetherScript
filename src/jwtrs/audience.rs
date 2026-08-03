//! `aud` reading: one string or an array of strings.
//!
//! One responsibility: normalise the `aud` claim into a bounded `Vec<String>`, and
//! decide whether it intersects the configured audiences.
//!
//! # Why both shapes must be handled
//!
//! RFC 7519 §4.1.3 defines `aud` as an array of `StringOrURI`, "or, in the special
//! case when there is one audience, ... a single string". Both forms are conforming
//! and real issuers emit both — Keycloak sends a bare string for a
//! single-audience token and an array once a second client is in scope. A verifier
//! that handles only one form breaks on ordinary configuration changes, and a
//! verifier that handles only the string form by *ignoring* arrays would accept
//! any array, which is worse.
//!
//! # Why a match is mandatory
//!
//! `aud` names the service the token was minted for. Skipping the check means
//! service B accepts a token the identity provider issued for service A — a
//! confused-deputy escalation, since A's token may carry roles B honours. So an
//! empty configured audience list rejects everything rather than waving tokens
//! through.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::audience::matches_any;
//!
//! let accepted = vec!["web-app".to_string()];
//! assert!(matches_any(&["web-app".to_string()], &accepted));
//! assert!(matches_any(&["other".to_string(), "web-app".to_string()], &accepted));
//! assert!(!matches_any(&["other".to_string()], &accepted));
//! assert!(!matches_any(&["web-app".to_string()], &[]));
//! ```

use std::collections::HashMap;

use crate::jwtrs::error_claims::ClaimError;
use crate::jwtrs::limits::MAX_AUDIENCES;
use crate::value::Value;

/// Read `aud` in either permitted shape.
///
/// # Arguments
///
/// * `members` — The authenticated payload object.
///
/// # Returns
///
/// The audiences, empty when `aud` is absent or null.
///
/// # Errors
///
/// [`ClaimError::AudienceNotStringOrArray`] for any other type or for an array
/// holding a non-string, and [`ClaimError::TooManyAudiences`] past
/// [`MAX_AUDIENCES`] — checked before the elements are copied, so an oversized
/// array is refused rather than allocated.
///
/// # Panics
///
/// Does not panic.
pub fn read(members: &HashMap<String, Value>) -> Result<Vec<String>, ClaimError> {
    match members.get("aud") {
        None | Some(Value::Nil) => Ok(Vec::new()),
        Some(Value::Str(text)) => Ok(vec![text.as_str().to_string()]),
        Some(Value::List(items)) => {
            let items = items.borrow();
            if items.len() > MAX_AUDIENCES {
                return Err(ClaimError::TooManyAudiences {
                    count: items.len(),
                    limit: MAX_AUDIENCES,
                });
            }
            items
                .iter()
                .map(|item| match item {
                    Value::Str(text) => Ok(text.as_str().to_string()),
                    other => Err(ClaimError::AudienceNotStringOrArray(other.type_name())),
                })
                .collect()
        }
        Some(other) => Err(ClaimError::AudienceNotStringOrArray(other.type_name())),
    }
}

/// Decide whether any presented audience is accepted.
///
/// # Arguments
///
/// * `presented` — The token's audiences.
/// * `accepted` — The configured audiences.
///
/// # Returns
///
/// `true` when the two sets intersect. Comparison is byte-exact: `aud` values are
/// `StringOrURI`, and URI-normalising them would let a crafted equivalent form
/// match an audience the issuer never wrote.
pub fn matches_any(presented: &[String], accepted: &[String]) -> bool {
    presented.iter().any(|value| accepted.contains(value))
}
