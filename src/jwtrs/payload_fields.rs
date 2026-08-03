//! Typed reads of individual payload members.
//!
//! One responsibility: the string and number accessors every claim rule shares.
//! Split out so [`crate::jwtrs::claims`] holds the claim *set* and this file holds
//! the JSON-to-Rust conversions.
//!
//! # Why absent and wrong-typed are different errors
//!
//! An absent `exp` means the issuer minted an eternal token; a string `exp` means
//! something upstream serialised a number as text. The first is a policy problem
//! and the second is a bug, so they get [`ClaimError::Missing`] and
//! [`ClaimError::NotNumber`] rather than one shared "bad claim".

use std::collections::HashMap;

use crate::jwtrs::error_claims::ClaimError;
use crate::value::Value;

/// Read an optional string claim.
///
/// # Arguments
///
/// * `members` — The authenticated payload object.
/// * `name` — The claim name.
///
/// # Returns
///
/// `Ok(None)` when absent or null, `Ok(Some(text))` when a string.
///
/// # Errors
///
/// [`ClaimError::NotString`] when present with any other JSON type.
pub(crate) fn optional_str(
    members: &HashMap<String, Value>,
    name: &'static str,
) -> Result<Option<String>, ClaimError> {
    match members.get(name) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Str(text)) => Ok(Some(text.as_str().to_string())),
        Some(other) => Err(ClaimError::NotString {
            name,
            found: other.type_name(),
        }),
    }
}

/// Read a required string claim.
///
/// # Errors
///
/// [`ClaimError::Missing`] when absent or null, [`ClaimError::NotString`] when
/// present with the wrong type.
pub(crate) fn required_str(
    members: &HashMap<String, Value>,
    name: &'static str,
) -> Result<String, ClaimError> {
    optional_str(members, name)?.ok_or(ClaimError::Missing(name))
}

/// Read an optional numeric-date claim as whole seconds since the Unix epoch.
///
/// JSON numbers reach the in-tree parser as either `Int` or `Float`, and RFC 7519
/// §2 allows a non-integer `NumericDate`, so both are accepted. A float is
/// truncated toward zero rather than rounded, so a fractional `exp` never gains a
/// sub-second of extra life.
///
/// # Errors
///
/// [`ClaimError::NotNumber`] when present but not numeric.
pub(crate) fn optional_secs(
    members: &HashMap<String, Value>,
    name: &'static str,
) -> Result<Option<i64>, ClaimError> {
    match members.get(name) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Int(value)) => Ok(Some(*value)),
        Some(Value::Float(value)) => Ok(Some(value.trunc() as i64)),
        Some(other) => Err(ClaimError::NotNumber {
            name,
            found: other.type_name(),
        }),
    }
}

/// Read a required numeric-date claim.
///
/// # Errors
///
/// [`ClaimError::Missing`] when absent — used for `exp`, which must never be
/// treated as "no expiry" — and [`ClaimError::NotNumber`] when non-numeric.
pub(crate) fn required_secs(
    members: &HashMap<String, Value>,
    name: &'static str,
) -> Result<i64, ClaimError> {
    optional_secs(members, name)?.ok_or(ClaimError::Missing(name))
}
