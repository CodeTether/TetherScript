//! Argument coercion for the password built-ins.
//!
//! Split from `password_install` to respect the 50-line file limit. Wrong-type
//! arguments name the offending parameter, so a script error points at the call
//! site rather than surfacing as a generic failure.

use super::password_ops::{needs_rehash, verify};
use crate::value::Value;

/// Coerce a str argument, naming the parameter when the type is wrong.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok(text.to_string()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Verify a password against a stored hash.
pub(super) fn verify_args(args: &[Value]) -> Result<Value, String> {
    let password = str_arg(&args[0], "password_verify: password")?;
    let encoded = str_arg(&args[1], "password_verify: encoded")?;
    verify(&password, &encoded).map(Value::Bool)
}

/// Decide whether a stored hash is below the required cost.
pub(super) fn needs_rehash_args(args: &[Value]) -> Result<Value, String> {
    let encoded = str_arg(&args[0], "password_needs_rehash: encoded")?;
    let minimum = match &args[1] {
        Value::Int(int) => *int,
        other => {
            return Err(format!(
                "password_needs_rehash: min_iterations must be int, got {}",
                other.type_name()
            ))
        }
    };
    needs_rehash(&encoded, minimum).map(Value::Bool)
}
