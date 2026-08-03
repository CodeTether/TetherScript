//! Fixed-window bucket key derivation for the durable rate limiter.
//!
//! # Why a derived key instead of in-process state
//!
//! The existing token bucket (`bucket_take`) hands its state back to the caller, so
//! it lives in one process's memory: it resets on restart and two workers behind a
//! load balancer each grant a full allowance. Deriving a key from the subject and
//! the *window index* fixes both — every process computing the same key increments
//! the same counter, and the counter survives a restart because Redis holds it.
//!
//! # The window index
//!
//! `index = now_secs / window_secs`, integer division. Every instant inside one
//! window maps to the same index regardless of clock skew below the window size, so
//! the key is stable within a window and changes exactly at the boundary. Including
//! `window_secs` in the key means a configuration change starts fresh counters
//! rather than inheriting a differently-sized window's count.
//!
//! Negative `now_secs` (pre-1970 clocks) is rejected: Rust's integer division
//! truncates toward zero, so `-1 / 60 == 0` would share a bucket with `+59`.

use super::sessionstore_validate::{component, SEP};
use super::sessionstore_window::require_window;

/// Derive the fixed-window bucket key.
///
/// # Arguments
///
/// * `prefix` — Namespace, e.g. `"rl"`.
/// * `subject` — Untrusted throttling subject: an IP, API key, or user id.
/// * `window_secs` — Window width in seconds; must be positive.
/// * `now_secs` — Current Unix time in seconds; must not be negative.
///
/// # Returns
///
/// `"<prefix>:<subject>:<window_secs>:<index>"`.
///
/// # Errors
///
/// Returns a named error when a component is empty, contains `:` or a control
/// character, when `window_secs` is not positive, or when `now_secs` is negative.
///
/// # Examples
///
/// ```rust,ignore
/// assert_eq!(derive("rl", "ip", 60, 125).unwrap(), "rl:ip:60:2");
/// assert!(derive("rl", "a:b", 60, 0).is_err());
/// ```
pub(super) fn derive(
    prefix: &str,
    subject: &str,
    window_secs: i64,
    now_secs: i64,
) -> Result<String, String> {
    let label = "ratelimit_window_key";
    component(&format!("{label}: prefix"), prefix)?;
    component(&format!("{label}: subject"), subject)?;
    require_window(label, window_secs)?;
    if now_secs < 0 {
        return Err(format!(
            "{label}: now_secs must not be negative, got {now_secs}"
        ));
    }
    let index = now_secs / window_secs;
    Ok(format!(
        "{prefix}{SEP}{subject}{SEP}{window_secs}{SEP}{index}"
    ))
}
