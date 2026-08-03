//! Argument coercion for channel built-ins.
//!
//! Built-in arguments arrive as dynamic values, so every coercion names both the
//! built-in and the offending type. "Error" is not an error message; a script
//! author must be able to see which argument of which call was wrong.

use crate::value::Value;

/// Read a non-negative capacity argument.
///
/// # Errors
///
/// Returns `Err` when the value is not an int or is negative.
pub(super) fn capacity(value: &Value, operation: &str) -> Result<usize, String> {
    match value {
        Value::Int(count) if *count >= 0 => Ok(*count as usize),
        Value::Int(count) => Err(format!("{operation}: capacity must not be negative: {count}")),
        other => Err(format!(
            "{operation}: capacity must be an int, got {}",
            other.type_name()
        )),
    }
}

/// Read a channel handle argument.
///
/// # Errors
///
/// Returns `Err` when the value is not an int.
pub(super) fn handle(value: &Value, operation: &str) -> Result<i64, String> {
    match value {
        Value::Int(handle) => Ok(*handle),
        other => Err(format!(
            "{operation}: channel handle must be an int, got {}",
            other.type_name()
        )),
    }
}

/// Read a diagnostic channel name argument.
///
/// # Errors
///
/// Returns `Err` when the value is not a str.
pub(super) fn name(value: &Value, operation: &str) -> Result<String, String> {
    match value {
        Value::Str(name) => Ok(name.as_str().to_string()),
        other => Err(format!(
            "{operation}: channel name must be a str, got {}",
            other.type_name()
        )),
    }
}
