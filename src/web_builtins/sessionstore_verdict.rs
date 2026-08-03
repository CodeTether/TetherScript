//! The fixed-window verdict: allowed, remaining, reset_at, retry_after_secs.
//!
//! # What `count` means
//!
//! `count` is the counter value **including** the request being judged — that is,
//! the reply Redis gives to `INCR`. So the first request of a window is judged with
//! `count = 1`, and `allowed = count <= limit`. Passing `0` describes a window in
//! which nothing has been counted yet and yields `allowed` with the full allowance
//! remaining, which is what a read-only probe wants.
//!
//! Fixing the meaning matters: an off-by-one here is a limiter that admits `limit+1`
//! or rejects the last legitimate request, and both look like a working limiter.
//!
//! `remaining` is clamped at zero rather than going negative, because it is served
//! to clients as `X-RateLimit-Remaining`, where a negative number is not meaningful.
//!
//! Read [`super::sessionstore_window`] for why the true burst ceiling is `2 x limit`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::sessionstore_window::require_window;
use crate::value::Value;

/// Judge one request against a fixed-window counter.
///
/// # Arguments
///
/// * `count` — Counter value including this request; must not be negative.
/// * `limit` — Requests permitted per window; must be positive.
/// * `window_secs` — Window width in seconds; must be positive.
/// * `now_secs` — Current Unix time in seconds; must not be negative.
///
/// # Returns
///
/// A map with `allowed` (bool), `remaining` (int), `reset_at` (int, Unix seconds at
/// the window's end) and `retry_after_secs` (int, 0 when allowed).
///
/// # Errors
///
/// Returns a named error when any argument is out of range.
///
/// # Examples
///
/// ```rust,ignore
/// let verdict = judge(3, 5, 60, 125).unwrap();
/// assert!(matches!(verdict, crate::value::Value::Map(_)));
/// ```
pub(super) fn judge(
    count: i64,
    limit: i64,
    window_secs: i64,
    now_secs: i64,
) -> Result<Value, String> {
    let label = "ratelimit_window_verdict";
    require_window(label, window_secs)?;
    check_range(label, count, limit, now_secs)?;
    let reset_at = (now_secs / window_secs + 1) * window_secs;
    let allowed = count <= limit;
    // At least 1 when denied: a Retry-After of 0 invites an immediate retry that is
    // certain to be rejected again.
    let retry = if allowed {
        0
    } else {
        (reset_at - now_secs).max(1)
    };
    Ok(fields(allowed, (limit - count).max(0), reset_at, retry))
}

/// Reject counts, limits, and clocks that cannot describe a real window.
fn check_range(label: &str, count: i64, limit: i64, now_secs: i64) -> Result<(), String> {
    if count < 0 {
        return Err(format!("{label}: count must not be negative, got {count}"));
    }
    if limit <= 0 {
        return Err(format!("{label}: limit must be positive, got {limit}"));
    }
    if now_secs < 0 {
        return Err(format!(
            "{label}: now_secs must not be negative, got {now_secs}"
        ));
    }
    Ok(())
}

/// Assemble the verdict map, keeping the script-visible shape in one place.
fn fields(allowed: bool, remaining: i64, reset_at: i64, retry_after_secs: i64) -> Value {
    let mut out = HashMap::new();
    out.insert("allowed".to_string(), Value::Bool(allowed));
    out.insert("remaining".to_string(), Value::Int(remaining));
    out.insert("reset_at".to_string(), Value::Int(reset_at));
    out.insert("retry_after_secs".to_string(), Value::Int(retry_after_secs));
    Value::Map(Rc::new(RefCell::new(out)))
}
