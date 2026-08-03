//! Wording for [`ShapeError`].
//!
//! One responsibility: render the stage-one refusals. Each message names the
//! segment and the concrete value, because "malformed token" tells an operator
//! nothing about whether the client is broken, the proxy is truncating headers,
//! or someone is probing the verifier.

use crate::jwtrs::error_shape::ShapeError;

/// Render a shape-stage rejection.
///
/// # Arguments
///
/// * `err` — The refusal to describe.
///
/// # Returns
///
/// A one-line message prefixed `jwtrs: `, naming the failed check.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn shape_text(err: &ShapeError) -> String {
    match err {
        ShapeError::TokenTooLarge { bytes, limit } => {
            format!("jwtrs: token is {bytes} bytes; limit is {limit}")
        }
        ShapeError::WrongSegmentCount(count) => format!(
            "jwtrs: expected 3 dot-separated segments, got {count}; \
             a 2-segment token is the unsecured form and is never accepted"
        ),
        ShapeError::EmptySegment(segment) => format!("jwtrs: `{segment}` segment is empty"),
        ShapeError::Base64 { segment, reason } => {
            format!("jwtrs: `{segment}` is not unpadded base64url: {reason}")
        }
        ShapeError::NotUtf8(segment) => format!("jwtrs: `{segment}` is not valid UTF-8"),
        ShapeError::MalformedJson { segment, detail } => {
            format!("jwtrs: `{segment}` is not valid JSON: {detail}")
        }
        ShapeError::NotAnObject { segment, found } => {
            format!("jwtrs: `{segment}` must be a JSON object, got {found}")
        }
        ShapeError::MissingAlg => "jwtrs: header is missing `alg`".to_string(),
        ShapeError::AlgNotString(found) => {
            format!("jwtrs: header `alg` must be a string, got {found}")
        }
        ShapeError::AlgNone => "jwtrs: header `alg` is `none`; unsecured JWS is never accepted"
            .to_string(),
        ShapeError::AlgMismatch { got, expected } => format!(
            "jwtrs: header `alg` is `{got}` but this verifier is pinned to `{expected}`"
        ),
        ShapeError::TypMismatch { got, expected } => {
            format!("jwtrs: header `typ` is `{got}`, expected `{expected}`")
        }
    }
}
