//! Human-readable rendering of [`JwksError`].
//!
//! One responsibility: the wording. Split from [`crate::jwks::error`] so adding a
//! variant and rewording a message are separate edits to separate files, and so
//! neither grows past the file limit.
//!
//! Every message names the thing that went wrong — the `kid` requested, the
//! algorithm asked for, the candidates that tied — because "no suitable key" with
//! no further detail is indistinguishable from a key-rotation lag, a `use: "enc"`
//! misconfiguration, and a typo in the realm name.

use std::fmt;

use crate::jwks::error::JwksError;

impl fmt::Display for JwksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedJson(detail) => write!(f, "jwks: malformed JSON: {detail}"),
            Self::DocumentTooLarge { bytes, limit } => {
                write!(f, "jwks: document is {bytes} bytes; limit is {limit}")
            }
            Self::NotAnObject(found) => {
                write!(f, "jwks: document must be a JSON object, got {found}")
            }
            Self::MissingKeys => write!(f, "jwks: document has no `keys` member"),
            Self::KeysNotArray(found) => write!(f, "jwks: `keys` must be an array, got {found}"),
            Self::TooManyKeys { count, limit } => {
                write!(f, "jwks: document has {count} keys; limit is {limit}")
            }
            Self::UnknownKid { kid, available } => write!(
                f,
                "jwks: no usable key with kid `{kid}`; usable kids: [{}]",
                available.join(", ")
            ),
            Self::NoSuitableKey { alg } => {
                write!(f, "jwks: no usable key is suitable for alg `{alg}`")
            }
            Self::AmbiguousKey { alg, candidates } => write!(
                f,
                "jwks: token carries no kid and {} keys are suitable for alg `{alg}` \
                 ([{}]); refusing to guess",
                candidates.len(),
                candidates.join(", ")
            ),
            Self::UnsuitableKey { kid, reason } => {
                write!(f, "jwks: key `{kid}` cannot be used: {reason}")
            }
            Self::UnsupportedAlgorithm(alg) => write!(
                f,
                "jwks: `{alg}` is not an RSA signature algorithm supported here"
            ),
        }
    }
}

impl std::error::Error for JwksError {}
