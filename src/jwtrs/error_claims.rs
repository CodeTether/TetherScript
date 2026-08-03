//! Failures decided from the token's *claims*, after the signature is verified.
//!
//! One responsibility: name the refusals that only make sense once the payload is
//! known to be authentic — issuer, audience, time window, and Keycloak's role
//! containers.
//!
//! # Why these are a separate type from [`ShapeError`](crate::jwtrs::error_shape::ShapeError)
//!
//! A `ClaimError` can only be produced downstream of signature verification. That
//! is not a convention: the code paths that construct these variants are reached
//! only from [`Authenticated`](crate::jwtrs::authenticated::Authenticated), which
//! cannot exist until a verifier has accepted the signature.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::error_claims::ClaimError;
//!
//! let err = ClaimError::Expired { exp: 100, now: 300, skew: 60 };
//! assert!(matches!(err, ClaimError::Expired { .. }));
//! ```

/// A refusal decided from an authenticated payload's claims.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimError {
    /// A claim this module requires was absent or null.
    Missing(&'static str),
    /// A claim was present but was not a string where a string is required.
    NotString {
        name: &'static str,
        found: &'static str,
    },
    /// A claim was present but was not a number where a number is required.
    NotNumber {
        name: &'static str,
        found: &'static str,
    },
    /// `iss` did not equal the configured issuer.
    IssuerMismatch { got: String, expected: String },
    /// `aud` was neither a string nor an array of strings.
    AudienceNotStringOrArray(&'static str),
    /// `aud` held more than [`crate::jwtrs::limits::MAX_AUDIENCES`] entries.
    TooManyAudiences { count: usize, limit: usize },
    /// No `aud` entry matched any configured audience.
    AudienceMismatch {
        got: Vec<String>,
        expected: Vec<String>,
    },
    /// `exp` had passed, even after adding the skew tolerance.
    Expired { exp: i64, now: i64, skew: i64 },
    /// `nbf` was in the future, even after subtracting the skew tolerance.
    NotYetValid { nbf: i64, now: i64, skew: i64 },
    /// A role container such as `realm_access` was present but not an object.
    RolesContainerNotObject { scope: String, found: String },
    /// A `roles` member was present but not an array.
    RolesNotArray { scope: String, found: String },
    /// A role array held a non-string element.
    RoleNotString { scope: String, found: String },
    /// A role array held more than [`crate::jwtrs::limits::MAX_ROLES`] entries.
    TooManyRoles {
        scope: String,
        count: usize,
        limit: usize,
    },
    /// `resource_access` held more than
    /// [`crate::jwtrs::limits::MAX_RESOURCE_CLIENTS`] clients.
    TooManyResourceClients { count: usize, limit: usize },
}
