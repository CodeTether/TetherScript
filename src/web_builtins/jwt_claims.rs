//! Time-based claim validation for `exp` and `nbf`.
//!
//! Both claims are optional; when present they are enforced. Absent claims are
//! not an error, matching RFC 7519, which leaves their presence to the issuer.

use crate::system::time_now_ms;
use crate::value::Value;

/// Read a numeric claim as seconds since the Unix epoch.
///
/// JSON numbers may decode as either int or float, so both are accepted. A
/// non-numeric claim is a malformed token rather than a missing one.
fn seconds(claims: &Value, name: &str) -> Result<Option<i64>, String> {
    let Value::Map(map) = claims else {
        return Err("jwt: payload must be a JSON object".into());
    };
    match map.borrow().get(name) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Int(value)) => Ok(Some(*value)),
        Some(Value::Float(value)) => Ok(Some(*value as i64)),
        Some(other) => Err(format!(
            "jwt: `{name}` claim must be a number, got {}",
            other.type_name()
        )),
    }
}

/// Current wall-clock time in seconds since the Unix epoch.
fn now_seconds() -> i64 {
    match time_now_ms() {
        Value::Int(ms) => ms / 1000,
        _ => 0,
    }
}

/// Reject a token that has expired or is not yet valid.
///
/// # Arguments
///
/// * `claims` — Decoded payload map.
///
/// # Returns
///
/// `Ok(())` when both claims are absent or satisfied.
///
/// # Errors
///
/// Returns `jwt: token expired` past `exp`, `jwt: token not yet valid` before
/// `nbf`, or a named error when either claim is present but not numeric. No
/// leeway is applied: a caller wanting clock skew tolerance should widen the
/// claim itself rather than have every verifier silently accept stale tokens.
pub(super) fn validate(claims: &Value) -> Result<(), String> {
    let now = now_seconds();
    if let Some(exp) = seconds(claims, "exp")? {
        if now >= exp {
            return Err(format!("jwt: token expired at {exp}, now {now}"));
        }
    }
    if let Some(nbf) = seconds(claims, "nbf")? {
        if now < nbf {
            return Err(format!("jwt: token not yet valid until {nbf}, now {now}"));
        }
    }
    Ok(())
}
