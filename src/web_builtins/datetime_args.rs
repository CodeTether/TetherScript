//! Argument coercion and the clock, shared by the registration module.
//!
//! Split from `datetime_install.rs` for the line limit. Type errors name the
//! built-in and the offending type so a script author sees which call was wrong.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::value::Value;

/// Current Unix seconds.
///
/// Mirrors how `time_now_ms` in `src/system.rs` reads the clock, including
/// treating a pre-epoch system clock as 0 rather than panicking.
///
/// # Returns
///
/// Whole seconds since 1970-01-01T00:00:00Z.
pub(super) fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs().min(i64::MAX as u64) as i64)
        .unwrap_or(0)
}

/// Coerce an int argument.
///
/// # Arguments
///
/// * `value` — Script-supplied value.
/// * `label` — Built-in and parameter name, used in the error.
///
/// # Returns
///
/// The integer.
///
/// # Errors
///
/// Returns an error naming `label` and the actual type.
pub(super) fn int_arg(value: &Value, label: &str) -> Result<i64, String> {
    match value {
        Value::Int(seconds) => Ok(*seconds),
        other => Err(format!("{label} must be int, got {}", other.type_name())),
    }
}

/// Coerce a str argument.
///
/// # Arguments
///
/// * `value` — Script-supplied value.
/// * `label` — Built-in and parameter name, used in the error.
///
/// # Returns
///
/// The string.
///
/// # Errors
///
/// Returns an error naming `label` and the actual type.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}
