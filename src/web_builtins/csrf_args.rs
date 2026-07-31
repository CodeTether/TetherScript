//! Argument coercion for the CSRF built-ins.
//!
//! Kept separate so [`super::csrf`] stays a registration list and the type errors
//! all read the same way.

use crate::value::Value;

use super::csrf_sign;

pub(super) fn token(args: &[Value]) -> Result<Value, String> {
    let secret = str_arg(&args[0], "csrf_token: secret")?;
    let ttl = int_arg(&args[1], "csrf_token: ttl_seconds")?;
    csrf_sign::token(&secret, ttl)
}

pub(super) fn verify(args: &[Value]) -> Result<Value, String> {
    let token = str_arg(&args[0], "csrf_verify: token")?;
    let secret = str_arg(&args[1], "csrf_verify: secret")?;
    csrf_sign::verify(&token, &secret)
}

pub(super) fn claims(args: &[Value]) -> Result<Value, String> {
    let token = str_arg(&args[0], "csrf_claims: token")?;
    csrf_sign::claims(&token)
}

fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

fn int_arg(value: &Value, label: &str) -> Result<i64, String> {
    match value {
        Value::Int(int) => Ok(*int),
        other => Err(format!("{label} must be int, got {}", other.type_name())),
    }
}
