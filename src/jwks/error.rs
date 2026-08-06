//! Error type for JWKS document parsing and key selection.
//!
//! One responsibility: name the ways a *document* or a *selection* can fail.
//! Per-key rejections are deliberately **not** variants here — an unusable key is
//! skipped with a reason string rather than failing the document, because a realm
//! legitimately publishes keys of several types. See
//! [`JwkSet::skipped`](crate::jwks::keyset::JwkSet::skipped).
//!
//! The [`std::fmt::Display`] implementation lives in
//! `crate::jwks::error_display` so this file holds the shape and that one holds
//! the wording.

/// A JWKS document- or selection-level failure.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::error::JwksError;
///
/// let error = JwksError::MalformedJson("unexpected end of input at byte 3".into());
/// match error {
///     JwksError::MalformedJson(detail) => assert!(detail.contains("byte 3")),
///     other => panic!("unexpected variant: {other}"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JwksError {
    /// The bytes were not valid JSON. Carries the in-tree parser's own message.
    MalformedJson(String),
    /// The document exceeded [`MAX_DOCUMENT_BYTES`](crate::jwks::limits::MAX_DOCUMENT_BYTES).
    DocumentTooLarge { bytes: usize, limit: usize },
    /// The top level parsed, but is not a JSON object. Carries the type found.
    NotAnObject(String),
    /// The top-level object has no `keys` member.
    MissingKeys,
    /// The `keys` member is present but is not an array. Carries the type found.
    KeysNotArray(String),
    /// The `keys` array exceeded [`MAX_KEYS`](crate::jwks::limits::MAX_KEYS).
    TooManyKeys { count: usize, limit: usize },
    /// No usable key carried the requested `kid`. Lists the ids that were usable.
    UnknownKid { kid: String, available: Vec<String> },
    /// No `kid` was requested and no usable key is suitable for the algorithm.
    NoSuitableKey { alg: String },
    /// No `kid` was requested and several usable keys are suitable, so no choice
    /// can be made without guessing. Lists the candidate ids.
    AmbiguousKey {
        alg: String,
        candidates: Vec<String>,
    },
    /// A `kid` matched, but that key is not suitable for the algorithm.
    UnsuitableKey { kid: String, reason: String },
    /// The requested algorithm is not an RSA signature algorithm this module
    /// implements. Refused up front so an `HS256` request can never reach a key.
    UnsupportedAlgorithm(String),
}
