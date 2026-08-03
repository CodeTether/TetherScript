//! Reading individual header members.
//!
//! One responsibility: the two member accessors the header needs. Split from
//! [`crate::jwtrs::header`] so that file is the policy and this one is the lookup.
//!
//! # Why `read_alg` takes the expected name as an argument
//!
//! It compares and returns; it never chooses. The expected name arrives from
//! [`ValidationConfig`](crate::jwtrs::config::ValidationConfig), so there is no way
//! to call this function without having already decided which algorithm is
//! acceptable.

use std::collections::HashMap;

use crate::jwtrs::error_shape::ShapeError;
use crate::value::Value;

/// Read `alg` and require it to equal `expected`.
///
/// # Arguments
///
/// * `members` — The decoded header object.
/// * `expected` — The algorithm name the verifier is pinned to.
///
/// # Returns
///
/// The algorithm name, which is necessarily equal to `expected`.
///
/// # Errors
///
/// [`ShapeError::MissingAlg`] when absent or null, [`ShapeError::AlgNotString`]
/// when it is some other JSON type, [`ShapeError::AlgNone`] for the unsecured
/// `none` algorithm — checked by name so the error is specific — and
/// [`ShapeError::AlgMismatch`] for every other disagreement, which is where an
/// `HS256`-for-`RS256` confusion attempt lands.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn read_alg(
    members: &HashMap<String, Value>,
    expected: &'static str,
) -> Result<String, ShapeError> {
    let name = match members.get("alg") {
        None | Some(Value::Nil) => return Err(ShapeError::MissingAlg),
        Some(Value::Str(text)) => text.as_str().to_string(),
        Some(other) => return Err(ShapeError::AlgNotString(other.type_name())),
    };
    // `none` is matched case-insensitively: RFC 7518 registers it lowercase, but a
    // forged header is not obliged to be conforming, and `NONE` must not slip
    // through into a mismatch message that hides what was attempted.
    if name.eq_ignore_ascii_case("none") {
        return Err(ShapeError::AlgNone);
    }
    if name != expected {
        return Err(ShapeError::AlgMismatch {
            got: name,
            expected,
        });
    }
    Ok(name)
}

/// Read an optional string member.
///
/// # Arguments
///
/// * `members` — The decoded header object.
/// * `name` — The member to read, such as `kid` or `typ`.
///
/// # Returns
///
/// `Some(text)` when the member is a string, and `None` when it is absent, null,
/// or any non-string type. A non-string `kid` is treated as absent rather than as
/// an error: `kid` is only a hint for key selection, and a token carrying a
/// numeric `kid` still fails at selection, with a clearer error than a shape
/// complaint would give.
pub(crate) fn read_optional_str(members: &HashMap<String, Value>, name: &str) -> Option<String> {
    match members.get(name) {
        Some(Value::Str(text)) => Some(text.as_str().to_string()),
        _ => None,
    }
}
