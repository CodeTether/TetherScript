//! Failures decided from the token's *shape*, before any claim is read.
//!
//! One responsibility: name the refusals that happen while the token is still
//! entirely untrusted — size, segment count, base64url alphabet, UTF-8, JSON
//! shape, and the pinned-algorithm check.
//!
//! # Why these are a separate type from [`ClaimError`](crate::jwtrs::error_claims::ClaimError)
//!
//! Everything here is decided *before* the signature is checked, so it describes
//! a token that is malformed rather than a token that is forged or stale. The two
//! sets are never mixed, which is what makes the pipeline order legible in the
//! type of [`JwtError`](crate::jwtrs::error::JwtError).
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::error_shape::ShapeError;
//!
//! let err = ShapeError::WrongSegmentCount(2);
//! assert_eq!(err, ShapeError::WrongSegmentCount(2));
//! ```

/// A refusal decided from the token's shape alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeError {
    /// The compact serialization exceeded [`crate::jwtrs::limits::MAX_TOKEN_BYTES`].
    TokenTooLarge { bytes: usize, limit: usize },
    /// The token did not have exactly three dot-separated segments.
    WrongSegmentCount(usize),
    /// A segment was present but empty.
    EmptySegment(&'static str),
    /// A segment was not strict unpadded base64url.
    Base64 {
        segment: &'static str,
        reason: String,
    },
    /// A decoded segment was not valid UTF-8.
    NotUtf8(&'static str),
    /// A decoded segment was not valid JSON.
    MalformedJson {
        segment: &'static str,
        detail: String,
    },
    /// A decoded segment was valid JSON but not a JSON object.
    NotAnObject {
        segment: &'static str,
        found: &'static str,
    },
    /// The header had no `alg` member.
    MissingAlg,
    /// The header's `alg` was present but not a string.
    AlgNotString(&'static str),
    /// The header declared the unsecured `none` algorithm.
    AlgNone,
    /// The header's `alg` was not the algorithm the verifier was configured for.
    AlgMismatch {
        got: String,
        expected: &'static str,
    },
    /// The header's `typ` did not match the required value.
    TypMismatch { got: String, expected: String },
}
