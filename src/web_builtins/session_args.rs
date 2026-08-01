//! Argument coercion for the session built-ins.
//!
//! Split from [`super::session`] so the registration list stays readable and the
//! group keeps one obvious entry point. Every failure names the parameter, so a
//! script author sees which argument was wrong rather than a bare type error.

use super::session_sign;
use super::session_ttl;
use crate::value::Value;

/// Coerce `session_sign(payload, secret)`.
///
/// # Errors
///
/// Returns an error when `secret` is not a str, or the payload is not a map.
pub(super) fn sign(args: &[Value]) -> Result<Value, String> {
    let secret = str_arg(&args[1], "session_sign: secret")?;
    session_sign::sign(&args[0], &secret)
}

/// Coerce `session_verify(value, secret)`.
///
/// # Errors
///
/// Returns an error when either argument is not a str, or verification fails.
pub(super) fn verify(args: &[Value]) -> Result<Value, String> {
    let value = str_arg(&args[0], "session_verify: value")?;
    let secret = str_arg(&args[1], "session_verify: secret")?;
    session_sign::verify(&value, &secret)
}

/// Coerce `session_touch(payload, ttl_seconds)`.
///
/// # Errors
///
/// Returns an error when `ttl_seconds` is not an int, or the payload is not a map.
pub(super) fn touch(args: &[Value]) -> Result<Value, String> {
    let ttl = match &args[1] {
        Value::Int(value) => *value,
        other => {
            return Err(format!(
                "session_touch: ttl_seconds must be int, got {}",
                other.type_name()
            ))
        }
    };
    session_ttl::touch(&args[0], ttl)
}

/// Coerce a built-in argument to a string, naming the parameter on mismatch.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}
