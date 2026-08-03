//! Scalar config fields: `credentials` and `max_age`.
//!
//! A present-but-mistyped field is an error rather than being ignored. Silently
//! dropping `credentials = "true"` (a str, not a bool) would produce a policy the
//! author believes carries credentials and which quietly does not, and the
//! failure would show up as a browser dropping cookies with no server-side trace.

use std::collections::HashMap;

use super::cors_fields as key;
use crate::value::Value;

/// Read the `credentials` flag.
///
/// # Returns
///
/// The flag, defaulting to false — the safe setting, since credentialed CORS
/// exposes authenticated responses to another origin.
///
/// # Errors
///
/// Returns an error naming the actual type when present and not a bool.
pub(super) fn credentials(config: &HashMap<String, Value>) -> Result<bool, String> {
    match config.get(key::CREDENTIALS) {
        None | Some(Value::Nil) => Ok(false),
        Some(Value::Bool(flag)) => Ok(*flag),
        Some(other) => Err(format!(
            "cors_policy: `credentials` must be bool, got {}",
            other.type_name()
        )),
    }
}

/// Read the `max_age` preflight cache lifetime.
///
/// # Returns
///
/// `Some(seconds)` when set, else `None` so no `Access-Control-Max-Age` is
/// emitted and the browser uses its own default.
///
/// # Errors
///
/// Returns an error when the value is not an int, or is negative. A negative
/// lifetime has no meaning; browsers discard the header, so the author would
/// believe preflights were cached when every request paid for one.
pub(super) fn max_age(config: &HashMap<String, Value>) -> Result<Option<i64>, String> {
    match config.get(key::MAX_AGE) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Int(seconds)) if *seconds >= 0 => Ok(Some(*seconds)),
        Some(Value::Int(seconds)) => Err(format!(
            "cors_policy: `max_age` must not be negative, got {seconds}"
        )),
        Some(other) => Err(format!(
            "cors_policy: `max_age` must be int seconds, got {}",
            other.type_name()
        )),
    }
}
