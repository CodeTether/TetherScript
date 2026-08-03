//! Script argument coercion for the JWKS built-ins.
//!
//! One responsibility: turn dynamically-typed arguments into the Rust types the
//! concern modules want, naming the offending parameter when a type is wrong.

use crate::value::Value;

use super::{jwks_document, jwks_find, jwks_parts};

/// Coerce one argument to a `String`.
///
/// # Arguments
///
/// * `value` — Script value to read.
/// * `label` — Qualified parameter name, such as `jwks_parse: json`.
///
/// # Returns
///
/// The owned string contents.
///
/// # Errors
///
/// Returns a named error reporting the label and the actual type.
///
/// # Examples
///
/// ```tether
/// println(str(jwks_parse(42).is_err()))   // true
/// ```
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Coerce the `jwks_parse` argument, then parse the document.
///
/// # Errors
///
/// Propagates a coercion failure or any document/key validation error.
pub(super) fn parse(args: &[Value]) -> Result<Value, String> {
    jwks_document::parse(&str_arg(&args[0], "jwks_parse: json")?)
}

/// Coerce the `jwks_find` arguments, then select the key.
///
/// # Errors
///
/// Propagates a coercion failure or a `kid` miss.
pub(super) fn find(args: &[Value]) -> Result<Value, String> {
    jwks_find::find(&args[0], &str_arg(&args[1], "jwks_find: kid")?)
}

/// Coerce the `jwt_header` argument, then decode the header unverified.
///
/// # Errors
///
/// Propagates a coercion failure or any header decoding error.
pub(super) fn header(args: &[Value]) -> Result<Value, String> {
    jwks_parts::header(&str_arg(&args[0], "jwt_header: token")?)
}

/// Coerce the `jwt_rs256_parts` argument, then extract the signing material.
///
/// # Errors
///
/// Propagates a coercion failure, a malformed token, or a refused `alg`.
pub(super) fn rs256_parts(args: &[Value]) -> Result<Value, String> {
    jwks_parts::rs256_parts(&str_arg(&args[0], "jwt_rs256_parts: token")?)
}
