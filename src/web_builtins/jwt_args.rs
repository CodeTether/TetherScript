//! Script argument coercion for the JWT built-ins.
//!
//! Split from `jwt.rs` purely to keep that file inside the 50-line limit. Each
//! helper names the parameter it rejected, so a script author sees which argument
//! was wrong rather than a bare type error.

use crate::value::Value;

use super::jwt_sign;

/// Coerce one argument to a `String`.
///
/// # Arguments
///
/// * `value` — Script value to read.
/// * `label` — Qualified parameter name, such as `jwt_verify: secret`.
///
/// # Returns
///
/// The owned string contents.
///
/// # Errors
///
/// Returns a named error reporting the label and the actual type.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Coerce `jwt_sign` arguments, then sign.
///
/// # Errors
///
/// Propagates a coercion failure, or any signing error.
pub(super) fn sign(args: &[Value]) -> Result<Value, String> {
    let secret = str_arg(&args[1], "jwt_sign: secret")?;
    jwt_sign::sign(&args[0], &secret)
}

/// Coerce `jwt_verify` arguments, then verify.
///
/// # Errors
///
/// Propagates a coercion failure, or any verification error.
pub(super) fn verify(args: &[Value]) -> Result<Value, String> {
    let token = str_arg(&args[0], "jwt_verify: token")?;
    let secret = str_arg(&args[1], "jwt_verify: secret")?;
    jwt_sign::verify(&token, &secret)
}

/// Coerce the `jwt_decode_unverified` argument, then decode without checking.
///
/// # Errors
///
/// Propagates a coercion failure, or any decoding error.
pub(super) fn decode_unverified(args: &[Value]) -> Result<Value, String> {
    let token = str_arg(&args[0], "jwt_decode_unverified: token")?;
    jwt_sign::decode_unverified(&token)
}
