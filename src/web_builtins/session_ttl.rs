//! Session lifetime: refreshing and checking `exp`.
//!
//! The reference middleware uses a 7-day TTL with
//! `TtlExtensionPolicy::OnEveryRequest`, so a session that keeps being used keeps
//! living. `touch` is the port of that policy: a handler calls it per request and
//! re-signs the result.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

// The reference uses a 7-day TTL (604800 seconds), but `session_touch` takes the
// TTL explicitly rather than defaulting, so the lifetime is always visible at the
// call site and no constant is needed here.

/// Current wall-clock time in Unix seconds.
///
/// Derived from the same clock as the `time_now_ms` builtin so scripts comparing
/// the two cannot see them disagree.
pub(super) fn now_secs() -> i64 {
    match crate::system::time_now_ms() {
        Value::Int(millis) => millis.div_euclid(1000),
        _ => 0,
    }
}

/// Copy a payload with `exp` moved forward, leaving every other key untouched.
///
/// # Arguments
///
/// * `payload` — Existing session payload.
/// * `ttl` — Lifetime in seconds from now. Must be positive.
///
/// # Returns
///
/// A new map with a refreshed `exp`, and `iat` preserved if already present.
///
/// # Errors
///
/// Returns an error when `payload` is not a map or `ttl` is not positive.
pub(super) fn touch(payload: &Value, ttl: i64) -> Result<Value, String> {
    let Value::Map(map) = payload else {
        return Err(format!(
            "session_touch: payload must be map, got {}",
            payload.type_name()
        ));
    };
    if ttl <= 0 {
        return Err(format!(
            "session_touch: ttl_seconds must be positive, got {ttl}"
        ));
    }
    let mut refreshed: HashMap<String, Value> = map.borrow().clone();
    refreshed.insert("exp".into(), Value::Int(now_secs() + ttl));
    Ok(Value::Map(Rc::new(RefCell::new(refreshed))))
}

/// Report whether a payload's `exp` has passed.
///
/// # Arguments
///
/// * `payload` — Session payload to inspect.
///
/// # Returns
///
/// True when `exp` is at or before now. A payload with **no** `exp` is treated as
/// non-expiring and returns false, matching a session cookie that carries no
/// lifetime of its own.
///
/// # Errors
///
/// Returns an error when `payload` is not a map, or `exp` is present but not an
/// int — a string `exp` is a bug worth surfacing rather than silently ignoring.
pub(super) fn expired(payload: &Value) -> Result<Value, String> {
    let Value::Map(map) = payload else {
        return Err(format!(
            "session_expired: payload must be map, got {}",
            payload.type_name()
        ));
    };
    match map.borrow().get("exp") {
        None | Some(Value::Nil) => Ok(Value::Bool(false)),
        Some(Value::Int(exp)) => Ok(Value::Bool(*exp <= now_secs())),
        Some(other) => Err(format!(
            "session_expired: `exp` must be an int of Unix seconds, got {}",
            other.type_name()
        )),
    }
}
