//! Built-in bodies for the rate-limit half: bucket key and verdict.
//!
//! Both take `now_secs` from the caller rather than reading a clock. A limiter whose
//! time is an argument is testable at an exact window boundary, and the caller
//! already needs a clock to set the key's TTL, so two independent reads would be a
//! second source of truth.

use std::rc::Rc;

use super::sessionstore_args::{int_arg, str_arg};
use super::sessionstore_verdict::judge;
use super::sessionstore_windowkey::derive;
use crate::value::Value;

/// `ratelimit_window_key(prefix, subject, window_secs, now_secs)`.
///
/// # Arguments
///
/// * `args` — `[prefix: str, subject: str, window_secs: int, now_secs: int]`.
///
/// # Returns
///
/// The bucket key, identical in every process for the same window.
///
/// # Errors
///
/// Returns a named error on a wrong argument type, a component containing the key
/// separator, a non-positive window, or a negative clock.
pub(super) fn window_key(args: &[Value]) -> Result<Value, String> {
    let label = "ratelimit_window_key";
    let prefix = str_arg(&args[0], &format!("{label}: prefix"))?;
    let subject = str_arg(&args[1], &format!("{label}: subject"))?;
    let window = int_arg(&args[2], &format!("{label}: window_secs"))?;
    let now = int_arg(&args[3], &format!("{label}: now_secs"))?;
    Ok(Value::Str(Rc::new(derive(&prefix, &subject, window, now)?)))
}

/// `ratelimit_window_verdict(count, limit, window_secs, now_secs)`.
///
/// # Arguments
///
/// * `args` — `[count: int, limit: int, window_secs: int, now_secs: int]`.
///
/// # Returns
///
/// A map: `allowed`, `remaining`, `reset_at`, `retry_after_secs`.
///
/// # Errors
///
/// Returns a named error on a wrong argument type or an out-of-range value.
pub(super) fn verdict(args: &[Value]) -> Result<Value, String> {
    let label = "ratelimit_window_verdict";
    let count = int_arg(&args[0], &format!("{label}: count"))?;
    let limit = int_arg(&args[1], &format!("{label}: limit"))?;
    let window = int_arg(&args[2], &format!("{label}: window_secs"))?;
    let now = int_arg(&args[3], &format!("{label}: now_secs"))?;
    judge(count, limit, window, now)
}
